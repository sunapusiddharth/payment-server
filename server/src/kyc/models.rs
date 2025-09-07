// src/kyc/models.rs

use serde::{Deserialize, Serialize};
use validator::Validate;
use sqlx::types::Uuid;

#[derive(Debug, Deserialize, Validate)]
pub struct KycVerifyRequest {
    pub user_id: Uuid,
    #[validate(length(equal = 10))]
    pub pan: Option<String>,
    #[validate(length(equal = 12))]
    pub aadhaar: Option<String>,
    #[validate(length(min = 1))]
    pub name: String,
    pub dob: String, // YYYY-MM-DD
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct KycVerification {
    pub user_id: Uuid,
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct KycVerifyResponse {
    pub status: String,
    pub kyc_tier: String,
    pub message: String,
}