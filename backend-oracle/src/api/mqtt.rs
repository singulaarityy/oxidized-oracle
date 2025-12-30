use rumqttc::{MqttOptions, AsyncClient, QoS, Event, Packet};
use std::time::Duration;
use crate::api::endpoint::CryptoEnvelope;
use crate::crypto::verify::process_submit_data;
use rustls;
use rustls_native_certs;
use crate::config::AppConfig;
use std::sync::Arc;
use colored::*;

pub async fn run_mqtt_listener(config: Arc<AppConfig>) {
    
    let mut mqttoptions = MqttOptions::new(&config.mqtt_client_id, &config.mqtt_host, config.mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    
    mqttoptions.set_credentials(&config.mqtt_username, &config.mqtt_password);

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

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to topic
    client.subscribe(&config.mqtt_topic, QoS::AtLeastOnce).await.unwrap();

    println!("📡 MQTT Connected: {}:{}", config.mqtt_host, config.mqtt_port);
    println!("👂 Listening on '{}'...", config.mqtt_topic.cyan());

    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                match notification {
                    Event::Incoming(Packet::Publish(publish)) => {
                        println!("📨 Msg received on: {}", publish.topic.magenta());
                        
                        // Parse JSON payload
                        match serde_json::from_slice::<CryptoEnvelope>(&publish.payload) {
                            Ok(envelope) => {
                                // Clone config for each processing task if needed, or pass reference if async allows
                                // process_submit_data takes Arc<AppConfig>
                                match process_submit_data(envelope, config.clone(), client.clone()).await {
                                    Ok(msg) => println!("{} {}", "✅ Success:".green(), msg),
                                    Err(e) => println!("{} {}", "❌ Verification Failed:".red(), e),
                                }
                            }
                            Err(e) => println!("{} {}", "❌ JSON Parse Failed:".red(), e),
                        }
                    }
                    _ => {
                        // Ignore other events
                    }
                }
            }
            Err(e) => {
                println!("MQTT Loop Error: {:?}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
