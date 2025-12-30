use tokio;

// modules
mod solana;
mod crypto;
mod api;
mod config;

// --- BAGIAN UTAMA (Aplikasi) ---
#[tokio::main]
async fn main() {
    // Load .env variables
    dotenvy::dotenv().ok();

    // Load Config (Env + Wallet) ONCE
    let config = config::AppConfig::load();

    println!("🚀 Oracle Server Starting...");
    println!("📡 Connecting to MQTT Broker...");

    // Jalankan MQTT Listener
    api::mqtt::run_mqtt_listener(config).await;
}