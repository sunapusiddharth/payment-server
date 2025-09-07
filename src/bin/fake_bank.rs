// src/bin/fake_bank.rs

use axum::{Router, routing::{post, get}, extract::Path};
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let bank_service = std::sync::Arc::new(FakeBankService::new(pool));

    let app = Router::new()
        .route("/bank/link-account", post(bank::handlers::link_account))
        .route("/bank/balance/:user_id", get(bank::handlers::get_balance))
        .route("/bank/transfer", post(bank::handlers::transfer))
        .layer(Extension(bank_service));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3002));
    println!("Fake Bank service listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}