// src/models.rs
use serde::Deserialize;

// --- AMPLOP LUAR ---

#[derive(Debug, Deserialize)]
pub struct CryptoEnvelope {
    /// Public Key pengirim (Hex String)
    pub public_key: String,

    /// Tanda tangan digital (Hex String)
    pub signature: String,

    /// Payload mentah (JSON String yang di-stringify)
    /// Kita terima sebagai String dulu untuk verifikasi signature
    pub payload: String, 
}

// --- ISI DATA (Setelah Amplop Dibuka) ---

#[derive(Debug, Deserialize)]
pub struct SensorData {
    pub device_id: String,
    pub timestamp: i64,
    pub readings: SensorReadings,
}

#[derive(Debug, Deserialize)]
pub struct SensorReadings {
    pub temperature: f32,
    pub humidity: Option<f32>,
    pub air_quality: Option<u32>,
}