// src/payment/models.rs

use serde::{Deserialize, Serialize};
use validator::Validate;
use sqlx::types::Uuid;
use crate::auth::crypto::hash_mobile; // reuse from Auth

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PayByPhoneRequest {
    #[validate(regex = "MOBILE_REGEX")]
    pub to_mobile: String,

    #[validate(range(min = 1, max = 500_000))]
    pub amount: u64, // in paise

    #[validate(length(equal = 36))]
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PayByQrRequest {
    #[validate(length(min = 1))]
    pub qr_code: String, // e.g., "payment://user/550e8400-e29b-41d4-a716-446655440000"

    #[validate(range(min = 1, max = 500_000))]
    pub amount: u64,

    #[validate(length(equal = 36))]
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PaymentResponse {
    pub tx_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount: u64,
    pub status: PaymentStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum PaymentStatus {
    Success,
    Failed,
    Pending, // if async fraud check
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Daily limit exceeded")]
    DailyLimitExceeded,

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid QR code")]
    InvalidQrCode,

    #[error("Idempotency key already used")]
    DuplicateIdempotencyKey,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("Wallet error: {0}")]
    WalletError(#[from] crate::wallet::WalletError),

    #[error("Fraud check failed: {0}")]
    FraudCheckFailed(String),
}

// Regex for Indian mobile
const MOBILE_REGEX: &str = r"^\+91[6-9]\d{9}$";