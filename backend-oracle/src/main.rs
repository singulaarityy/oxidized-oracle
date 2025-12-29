use axum::{
    routing::post,
    Router,
};

// modules
mod solana;
mod crypto;
mod api;


// --- BAGIAN UTAMA (SERVER) ---
#[tokio::main]
async fn main() {
    // Setup Routing
    // Kalau ada POST ke /api/submit -> panggil handle_submit_data
    let app = Router::new()
        .route("/api/submit", post(crate::crypto::verify::handle_submit_data));

    // Nyalakan Server di port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Server Verifikator berjalan di port 3000...");
    axum::serve(listener, app).await.unwrap();
}