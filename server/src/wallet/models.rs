// src/models.rs

use serde::{Deserialize, Serialize};
use validator::Validate;
use sqlx::types::Uuid;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateWalletRequest {
    #[validate(length(min = 1, message = "user_id required"))]
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreditDebitRequest {
    #[validate(length(min = 1))]
    pub user_id: Uuid,

    #[validate(range(min = 1, max = 500_000))] // ₹5L max per tx
    pub amount: u64,

    #[validate(length(equal = 36))] // UUIDv4 as string
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct Wallet {
    pub user_id: Uuid,
    pub balance: i64,          // in paise (₹1 = 100 paise) — avoids float
    pub version: i32,          // for optimistic concurrency
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Idempotency key already used")]
    DuplicateIdempotencyKey,

    #[error("Wallet not found for user: {0}")]
    WalletNotFound(Uuid),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("Concurrency conflict - retry")]
    ConcurrencyConflict,
}