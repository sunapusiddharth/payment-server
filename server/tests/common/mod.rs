// tests/common/mod.rs
use sqlx::{PgPool, Postgres, Pool};
use redis::{Client, AsyncCommands};
use std::sync::Arc;

pub struct TestContext {
    pub db: PgPool,
    pub redis_client: Client,
}

impl TestContext {
    pub async fn new() -> Self {
        let db = PgPool::connect(&std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| "postgres://localhost/payment_system_test".to_string()))
            .await
            .expect("Failed to connect to test DB");

        let redis_client = Client::open("redis://127.0.0.1:6379").expect("Failed to connect to Redis");

        // Clear Redis before each test
        let mut conn = redis_client.get_async_connection().await.unwrap();
        let _: () = conn.flushall().await.unwrap();

        Self { db, redis_client }
    }

    pub async fn cleanup(&self) {
        // Truncate all tables
        sqlx::query!("TRUNCATE TABLE users, wallets, transaction_journal, daily_limits, idempotency_keys, refresh_tokens, fraud_flags, otp_store RESTART IDENTITY")
            .execute(&self.db)
            .await
            .unwrap();
    }
}

// Helper to generate UUID
pub fn new_uuid() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

// Helper to generate JWT
pub fn generate_jwt(user_id: &uuid::Uuid, secret: &str) -> String {
    crate::auth::crypto::create_jwt(user_id, None, secret, 3600).unwrap()
}