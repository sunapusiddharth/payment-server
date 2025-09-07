// src/bin/fraud_worker.rs

use payment_system::fraud::worker::FraudWorker;
use sqlx::PgPool;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let worker = FraudWorker::new(
        &std::env::var("NATS_URL").unwrap(),
        pool,
        std::env::var("ALERT_WEBHOOK_URL").unwrap(),
    )
    .await
    .unwrap();

    worker.start().await.unwrap();
}