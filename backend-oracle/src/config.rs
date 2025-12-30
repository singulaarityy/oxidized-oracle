use std::env;
use std::sync::Arc;
use solana_sdk::signature::Keypair;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use colored::*;

pub struct AppConfig {
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub mqtt_client_id: String,
    pub mqtt_topic: String,
    
    // Solana
    pub rpc_client: RpcClient,
    pub payer: Keypair,
    pub memo_program_id: String,
}

impl AppConfig {
    pub fn load() -> Arc<Self> {
        println!("{}", "🔄 Loading Config...".blue());
        
        let mqtt_host = env::var("MQTT_HOST").expect("MQTT_HOST required");
        let mqtt_port = env::var("MQTT_PORT").unwrap_or("8883".to_string()).parse().expect("Port must be a number");
        let mqtt_username = env::var("MQTT_USERNAME").expect("MQTT_USERNAME required");
        let mqtt_password = env::var("MQTT_PASSWORD").expect("MQTT_PASSWORD required");
        let mqtt_client_id = env::var("MQTT_CLIENT_ID").unwrap_or("oxidized-oracle".to_string());
        let mqtt_topic = env::var("MQTT_TOPIC").unwrap_or("oracle/submit".to_string());

        // Solana Setup
        let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or("https://api.devnet.solana.com".to_string());
        let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        // Load Wallet from Env
        println!("{}", "🔑 Reading Wallet from Env...".cyan());
        let wallet_key_str = env::var("SOLANA_WALLET_KEY").expect("SOLANA_WALLET_KEY required in .env");
        let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_key_str).expect("Invalid SOLANA_WALLET_KEY format (must be JSON array)");
        let payer = Keypair::from_bytes(&wallet_bytes).expect("Invalid Keypair bytes");

        let memo_program_id = env::var("SOLANA_MEMO_PROGRAM_ID")
            .unwrap_or("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo".to_string());

        println!("{}", "✅ Config Ready!".green().bold());

        Arc::new(Self {
            mqtt_host,
            mqtt_port,
            mqtt_username,
            mqtt_password,
            mqtt_client_id,
            mqtt_topic,
            rpc_client,
            payer,
            memo_program_id,
        })
    }
}
