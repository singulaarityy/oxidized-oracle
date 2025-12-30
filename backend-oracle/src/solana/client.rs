use solana_sdk::{
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use crate::config::AppConfig;
use colored::*;

pub fn catat_ke_blockchain(pesan_valid: &str, config: &AppConfig) -> Result<String, String> {
    println!("{}", "🔗 Connecting to Solana...".yellow());

    // 1. BUAT INSTRUKSI MEMO
    let memo_program_id = Pubkey::from_str(&config.memo_program_id).unwrap();
    let instruction = Instruction {
        program_id: memo_program_id,
        accounts: vec![],
        data: pesan_valid.as_bytes().to_vec(),
    };

    // 2. BUNGKUS KE TRANSAKSI
    let message = Message::new(
        &[instruction], 
        Some(&config.payer.pubkey()), 
    );

    let recent_blockhash = config.rpc_client.get_latest_blockhash()
        .map_err(|e| format!("Failed to get blockhash: {}", e))?;

    let transaction = Transaction::new(&[&config.payer], message, recent_blockhash);

    // 3. KIRIM
    println!("{}", "🚀 Sending tx...".cyan());
    match config.rpc_client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            let url = format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            Ok(url)
        }
        Err(e) => Err(format!("Failed to send to chain: {}", e)),
    }
}