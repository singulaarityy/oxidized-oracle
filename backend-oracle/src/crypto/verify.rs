use ed25519_dalek::{Signature, Verifier, PublicKey};
use hex;
use crate::api::endpoint::{CryptoEnvelope, SensorData};
use axum::{Json, http::StatusCode};

pub async fn handle_submit_data(Json(envelope): Json<CryptoEnvelope>) -> Result<String, StatusCode> {

    println!("📩 Menerima Amplop dari: {}", envelope.public_key);

    // LANGKAH A: DECODE HEX
    // Kita ubah string hex menjadi vector of bytes (Vec<u8>)
    let pub_key_bytes = hex::decode(&envelope.public_key)
        .map_err(|_| StatusCode::BAD_REQUEST)?; // Kalau gagal decode, return Error 400
    
    let sig_bytes = hex::decode(&envelope.signature)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // LANGKAH B: SIAPKAN ALAT VERIFIKASI
    // Ubah bytes menjadi objek PublicKey (KTP Pengirim) - ed25519-dalek v1
    let public_key = PublicKey::from_bytes(&pub_key_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Ubah bytes menjadi objek Signature (Tanda Tangan)
    // Ed25519 Signature panjangnya harus pas 64 bytes
    let signature = Signature::try_from(sig_bytes.as_slice())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // LANGKAH C: Cek Keaslian
    // "Apakah signature ini valid untuk payload (surat) ini?"
    match public_key.verify(envelope.payload.as_bytes(), &signature) {
        Ok(_) => {
            println!("✅ Tanda Tangan VALID!");
            
            // LANGKAH D: BUKA SURATNYA
            // Karena valid, kita aman mengubah payload string menjadi SensorData struct
            let data: SensorData = serde_json::from_str(&envelope.payload)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            println!("📊 Data Terbaca: Device={}, Suhu={}°C, Humidity={:?}%, AirQuality={:?}", 
                data.device_id, 
                data.readings.temperature,
                data.readings.humidity,
                data.readings.air_quality,
            );

            // LOG KE SOLANA
            let log_message = format!("Device: {}, Temp: {}C, Humidity: {}, AirQuality: {}", 
                data.device_id, 
                data.readings.temperature, 
                data.readings.humidity.map(|v: f32| v.to_string()).unwrap_or("N/A".to_string()),
                data.readings.air_quality.map(|v: u32| v.to_string()).unwrap_or("N/A".to_string())
            );
            
            tokio::spawn(async move {
                match crate::solana::client::catat_ke_blockchain(&log_message) {
                    Ok(url) => println!("🎉 SUKSES TERCATAT! Cek Explorer: {}", url),
                    Err(e) => println!("⚠️ Gagal mencatat ke chain: {}", e),
                }
            });

            Ok("Data diterima dan valid!".to_string())
        }
        Err(_) => {
            println!("❌ Tanda Tangan PALSU / Data Berubah!");
            Err(StatusCode::UNAUTHORIZED) // Return Error 401
        }
    }
}