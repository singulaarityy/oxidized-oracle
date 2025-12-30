use rumqttc::{MqttOptions, AsyncClient, QoS};
use serde::Serialize; // Removed Deserialize
use serde_json::json;
use ed25519_dalek::{Keypair, Signer};
use rand::rngs::OsRng;
use rand::Rng; 
use std::time::Duration;
use tokio::time;
use chrono::Utc;
use colored::*;
use rustls::{ClientConfig, RootCertStore, Certificate};
use rustls_native_certs::load_native_certs;
use std::sync::Arc;

// --- STRUKTUR DATA ---
#[derive(Debug, Serialize)]
struct CryptoEnvelope {
    public_key: String,
    signature: String,
    payload: String,
}

#[tokio::main]
async fn main() {
    println!("{}", "🤖 STUNTMAN: Starting Fake IoT Device...".blue().bold());

    // 1. SETUP MQTT
    let mut mqttoptions = MqttOptions::new("CLIENT_ID", "HOST_MQTT", 8883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_credentials("USERNAME_MQTT", "PASSWORD_MQTT");

    // TLS Setup (Wajib untuk HiveMQ Cloud)
    let mut root_store = RootCertStore::empty();
    for cert in load_native_certs().expect("Failed to load system certs") {
        root_store.add(&Certificate(cert.0)).unwrap();
    }
    let client_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    mqttoptions.set_transport(rumqttc::Transport::Tls(rumqttc::TlsConfiguration::Rustls(Arc::new(client_config))));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // 2. GENERATE IDENTITY (Device Secret Key)
    let mut csprng = OsRng;
    let keypair: Keypair = Keypair::generate(&mut csprng);
    println!("🔑 Device Public Key: {}", hex::encode(keypair.public.to_bytes()).yellow());

    // Subscribe to Feedback channel
    client.subscribe("oracle/response", QoS::AtLeastOnce).await.unwrap();

    // Spawn loop to handle MQTT events (Keepalive + Responses)
    tokio::spawn(async move {
        while let Ok(notification) = eventloop.poll().await {
            match notification {
                rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) => {
                    if publish.topic == "oracle/response" {
                        let payload = String::from_utf8_lossy(&publish.payload);
                        println!("{} Payload: {}", "📥 Received Confirmation:".cyan(), payload);
                    }
                }
                _ => {}
            }
        }
    });

    println!("{}", "📡 Connected to MQTT! Starting data stream...".green());

    loop {
        // 3. GENERATE RANDOM SENSOR DATA
        let mut rng = rand::thread_rng();
        let temp: f32 = rng.gen_range(20.0, 45.0); // Temp 20 - 45 C
        let hum: f32 = rng.gen_range(30.0, 90.0);  // Humidity 30 - 90 %
        let aq: u32 = rng.gen_range(0, 100);       // AQI 0 - 100

        let timestamp = Utc::now().timestamp();

        let sensor_data = json!({
            "device_id": "ESP32_SIMULATOR",
            "timestamp": timestamp,
            "readings": {
                "temperature": (temp * 10.0).round() / 10.0, // 1 decimal
                "humidity": (hum * 10.0).round() / 10.0,
                "air_quality": aq,
            }
        });

        let payload_str = sensor_data.to_string();
        
        // 4. SIGN DATA
        let signature = keypair.sign(payload_str.as_bytes());

        // 5. WRAP ENVELOPE
        let envelope = CryptoEnvelope {
            public_key: hex::encode(keypair.public.to_bytes()),
            signature: hex::encode(signature.to_bytes()),
            payload: payload_str.clone(),
        };

        let json_envelope = serde_json::to_string(&envelope).unwrap();

        // 6. PUBLISH TO MQTT
        let topic = "sensor/data";
        println!("📤 Sending: T={}°C, H={}% (ts: {})", temp as i32, hum as i32, timestamp);
        
        match client.publish(topic, QoS::AtLeastOnce, false, json_envelope).await {
            Ok(_) => {},
            Err(e) => println!("{} {}", "❌ Publish Failed:".red(), e),
        }

        time::sleep(Duration::from_secs(5)).await;
    }
}
