use rumqttc::{MqttOptions, AsyncClient, QoS, Event, Packet};
use std::time::Duration;
use crate::api::endpoint::CryptoEnvelope;
use crate::crypto::verify::process_submit_data;
use rustls;
use rustls_native_certs;

pub async fn run_mqtt_listener() {
    let host = std::env::var("MQTT_HOST").expect("MQTT_HOST wajib di .env");
    let port = std::env::var("MQTT_PORT")
        .unwrap_or_else(|_| "8883".to_string())
        .parse::<u16>()
        .expect("MQTT_PORT harus angka");
    let client_id = std::env::var("MQTT_CLIENT_ID").unwrap_or_else(|_| "oxidized-oracle-backend".to_string());
    
    let mut mqttoptions = MqttOptions::new(client_id, host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    
    // --- AUTHENTICATION ---
    let username = std::env::var("MQTT_USERNAME").expect("MQTT_USERNAME wajib di .env");
    let password = std::env::var("MQTT_PASSWORD").expect("MQTT_PASSWORD wajib di .env");
    mqttoptions.set_credentials(username, password);

    // --- TLS / SSL (Wajib untuk Port 8883) ---
    // Load sertifikat Root CA dari sistem operasi
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("Gagal memuat sertifikat sistem") {
        root_store.add(&rustls::Certificate(cert.0)).unwrap();
    }

    let transport = rumqttc::Transport::Tls(
        rumqttc::TlsConfiguration::Rustls(
            std::sync::Arc::new(
                rustls::ClientConfig::builder()
                    .with_safe_defaults()
                    .with_root_certificates(root_store)
                    .with_no_client_auth()
            )
        )
    );
    mqttoptions.set_transport(transport);

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to topic
    let topic = std::env::var("MQTT_TOPIC").unwrap_or_else(|_| "oracle/submit".to_string());
    client.subscribe(&topic, QoS::AtLeastOnce).await.unwrap();

    let host = std::env::var("MQTT_HOST").unwrap_or_default();
    let port = std::env::var("MQTT_PORT").unwrap_or_default();
    println!("📡 MQTT Listener Terhubung ke {}:{}", host, port);
    println!("👂 Menunggu pesan di topic '{}'...", topic);

    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                match notification {
                    Event::Incoming(Packet::Publish(publish)) => {
                        println!("📨 Pesan Masuk di topik: {}", publish.topic);
                        
                        // Parse JSON payload
                        match serde_json::from_slice::<CryptoEnvelope>(&publish.payload) {
                            Ok(envelope) => {
                                match process_submit_data(envelope).await {
                                    Ok(msg) => println!("✅ Sukses: {}", msg),
                                    Err(e) => println!("❌ Gagal Verifikasi: {}", e),
                                }
                            }
                            Err(e) => println!("❌ Gagal Parse JSON Envelope: {}", e),
                        }
                    }
                    _ => {
                        // Ignore other events
                    }
                }
            }
            Err(e) => {
                println!("Error dalam event loop MQTT: {:?}", e);
                // Kita coba tunggu sebentar sebelum reconnect (rumqttc biasanya handle reconnect sendiri di eventloop, tapi kalau fatal error perlu handle)
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
