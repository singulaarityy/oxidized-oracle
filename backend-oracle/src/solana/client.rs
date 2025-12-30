use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::{Signer, read_keypair_file},
    transaction::Transaction,
};
use std::str::FromStr;

// Hapus konstanta hardcoded
// const MEMO_PROGRAM_ID: &str = "Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo";

pub fn catat_ke_blockchain(pesan_valid: &str) -> Result<String, String> {
    println!("🔗 Memulai koneksi ke Solana...");

    // 1. KONEKSI KE RPC (Jaringan)
    let rpc_url = std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    // 2. LOAD DOMPET SERVER
    let home = std::env::var("HOME").unwrap(); // Tetap pakai HOME untuk fallback path kalau perlu, atau pakai full path dari env
    
    // Cek apakah SOLANA_WALLET_PATH absolute atau relative
    let configured_path = std::env::var("SOLANA_WALLET_PATH").unwrap_or_else(|_| "server-wallet.json".to_string());
    let wallet_path = if configured_path.starts_with("/") {
        configured_path
    } else {
        format!("{}/{}", home, configured_path)
    };

    let payer = read_keypair_file(&wallet_path)
        .map_err(|e| format!("Gagal baca wallet di {}: {}", wallet_path, e))?;

    // 3. BUAT INSTRUKSI MEMO
    let memo_program_id_str = std::env::var("SOLANA_MEMO_PROGRAM_ID")
        .unwrap_or_else(|_| "Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo".to_string());
    let memo_program_id = Pubkey::from_str(&memo_program_id_str).unwrap();
    let instruction = Instruction {
        program_id: memo_program_id,
        accounts: vec![],
        data: pesan_valid.as_bytes().to_vec(),
    };

    // 4. BUNGKUS KE TRANSAKSI
    let message = Message::new(
        &[instruction], 
        Some(&payer.pubkey()), 
    );

    let recent_blockhash = client.get_latest_blockhash()
        .map_err(|e| format!("Gagal dapat blockhash: {}", e))?;

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    // 5. KIRIM
    println!("🚀 Mengirim transaksi...");
    match client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            let url = format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            Ok(url)
        }
        Err(e) => Err(format!("Gagal kirim ke chain: {}", e)),
    }
}