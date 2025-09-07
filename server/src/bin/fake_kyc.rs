// src/bin/fake_kyc.rs

use axum::{Router, routing::post};
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let kyc_service = std::sync::Arc::new(FakeKycService::new(pool));

    let app = Router::new()
        .route("/kyc/verify", post(kyc::handlers::verify_kyc))
        .layer(Extension(kyc_service));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("Fake KYC service listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}