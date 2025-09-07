// src/fraud/models.rs

use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use chrono::DateTime;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FraudEvent {
    pub tx_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount: u64, // in paise
    pub device_fingerprint: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FraudFlag {
    pub tx_id: Uuid,
    pub risk_score: i32,
    pub reason: String,
    pub flagged_at: DateTime<chrono::Utc>,
    pub reviewed: bool,
    pub reviewed_at: Option<DateTime<chrono::Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum FraudError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("NATS error: {0}")]
    NatsError(#[from] nats::Error),

    #[error("Alert webhook error: {0}")]
    AlertError(#[from] reqwest::Error),
}