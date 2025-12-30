use tokio;

// modules
mod solana;
mod crypto;
mod api;

// --- BAGIAN UTAMA (Aplikasi) ---
#[tokio::main]
async fn main() {
    // Load .env variables
    dotenvy::dotenv().ok();

    println!("🚀 Server Oracle Memulai...");
    println!("📡 Menghubungkan ke MQTT Broker...");

    // Jalankan MQTT Listener
    // Fungsi ini akan loop forever
    api::mqtt::run_mqtt_listener().await;
}