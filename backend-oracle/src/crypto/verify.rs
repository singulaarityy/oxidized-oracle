use ed25519_dalek::{Signature, Verifier, PublicKey};
use hex;
use crate::api::endpoint::{CryptoEnvelope, SensorData};
use crate::config::AppConfig;
use std::sync::Arc;
use colored::*;
use chrono::TimeZone;

use rumqttc::{AsyncClient, QoS};
use serde_json::json;

pub async fn process_submit_data(envelope: CryptoEnvelope, config: Arc<AppConfig>, client: AsyncClient) -> Result<String, String> {

    println!("📩 Rx Envelope: {}", envelope.public_key.yellow());

    // DECODE HEX
    let pub_key_bytes = hex::decode(&envelope.public_key)
        .map_err(|_| "Invalid Public Key Hex".red().to_string())?;
    
    let sig_bytes = hex::decode(&envelope.signature)
        .map_err(|_| "Invalid Signature Hex".red().to_string())?;

    // VERIFY
    let public_key = PublicKey::from_bytes(&pub_key_bytes)
        .map_err(|_| "Invalid Public Key Bytes".red().to_string())?;

    let signature = Signature::try_from(sig_bytes.as_slice())
        .map_err(|_| "Invalid Signature Bytes".red().to_string())?;

    // CHECK SIGNATURE
    match public_key.verify(envelope.payload.as_bytes(), &signature) {
        Ok(_) => {
            println!("{}", "✅ Verified signature!".green().bold());
            
            // PARSE DATA
            let data: SensorData = serde_json::from_str(&envelope.payload)
                .map_err(|_| "Failed to Parse Payload JSON".red().to_string())?;

            // Format Timestamp
            let dt = chrono::Utc.timestamp_opt(data.timestamp, 0).single().unwrap_or_default();
            let ts_str = dt.format("%H:%M:%S").to_string();

            println!("📊 Data: [{}], Device={}, Temp={}°C, Hum={:?}, Air={:?}", 
                ts_str.blue(),
                data.device_id.cyan(), 
                data.readings.temperature,
                data.readings.humidity,
                data.readings.air_quality,
            );

            // LOG TO SOLANA
            let log_message = format!("Time: {}, Device: {}, Temp: {}C, Hum: {}, Air: {}", 
                data.timestamp,
                data.device_id, 
                data.readings.temperature, 
                data.readings.humidity.map(|v: f32| v.to_string()).unwrap_or("N/A".to_string()),
                data.readings.air_quality.map(|v: u32| v.to_string()).unwrap_or("N/A".to_string())
            );
            
            // Pass config clone to task
            let config_clone = config.clone();
            let client_clone = client.clone();
            
            tokio::spawn(async move {
                match crate::solana::client::catat_ke_blockchain(&log_message, &config_clone) {
                    Ok(url) => {
                        println!("{} {}", "🎉 Tx Sent:".green().bold(), url);
                        
                        // FEEDBACK LOOP
                        let response_payload = json!({
                            "device_id": data.device_id,
                            "status": "success",
                            "tx_url": url,
                            "timestamp": chrono::Utc::now().timestamp()
                        });
                        
                        // Publish Response
                        let _ = client_clone.publish(&config_clone.mqtt_response_topic, QoS::AtLeastOnce, false, response_payload.to_string()).await;
                    }
                    Err(e) => println!("{} {}", "⚠️ Solana Error:".red(), e),
                }
            });

            Ok("Data verified & processed!".to_string())
        }
        Err(_) => {
            println!("{}", "❌ Invalid Signature / Data Altered!".red().bold());
            Err("Unauthorized: Invalid Signature".to_string())
        }
    }
}