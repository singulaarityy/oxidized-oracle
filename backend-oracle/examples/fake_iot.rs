use ed25519_dalek::{Signer, Keypair};
use rand::rngs::OsRng;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
struct CryptoEnvelope {
    public_key: String,
    signature: String,
    payload: String,
}

#[tokio::main]
async fn main() {
    println!("🤖 STUNTMAN: Menyalakan IoT Palsu...");

    // 1. GENERATE KUNCI
    let mut csprng = OsRng;
    let keypair: Keypair = Keypair::generate(&mut csprng);

    // 2. SIAPKAN DATA SENSOR
    let sensor_data = json!({
        "device_id": "STUNTMAN_01",
        "timestamp": 17000000,
        "readings": {
            "temperature": -123.2,
            "humidity": 123.0,
        }
    });
    let payload_str = sensor_data.to_string(); 
    println!("📦 Payload Asli: {}", payload_str);

    // 3. TANDA TANGANI DATA
    let signature = keypair.sign(payload_str.as_bytes());
    
    // 4. BUNGKUS
    let envelope = CryptoEnvelope {
        public_key: hex::encode(keypair.public.to_bytes()),
        signature: hex::encode(signature.to_bytes()),      
        payload: payload_str,                              
    };

    println!("📨 Mengirim Amplop ke Server...");

    // 5. KIRIM VIA HTTP POST
    let client = reqwest::Client::new();
    // Pastikan portnya 3000 sesuai server main.rs Anda
    let res = client.post("http://localhost:3000/api/submit")
        .json(&envelope)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                println!("✅ SUKSES! Server menerima data kita.");
                println!("Respon Server: {}", response.text().await.unwrap());
            } else {
                println!("❌ DITOLAK! Server curiga.");
                println!("Status: {}", response.status());
            }
        }
        Err(e) => println!("💀 Gagal konek ke server: {}", e),
    }
}