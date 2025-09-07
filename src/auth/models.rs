// src/auth/models.rs

use serde::{Deserialize, Serialize};
use validator::Validate;
use sqlx::types::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(regex = "MOBILE_REGEX", message = "Invalid Indian mobile number")]
    pub mobile: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct VerifyOtpRequest {
    #[validate(regex = "MOBILE_REGEX")]
    pub mobile: String,

    #[validate(length(equal = 6, message = "OTP must be 6 digits"))]
    pub otp: String,

    #[serde(default)]
    pub device_fingerprint: Option<String>, // sent by client SDK
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64, // seconds
    pub user_id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub mobile_hash: String, // never store raw mobile
    pub device_fingerprint: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RefreshToken {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub device_fingerprint: Option<String>,
}

// Regex for Indian mobile: +91 followed by 10 digits starting with 6-9
const MOBILE_REGEX: &str = r"^\+91[6-9]\d{9}$";

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid OTP")]
    InvalidOtp,

    #[error("OTP expired or not found")]
    OtpExpired,

    #[error("User not found")]
    UserNotFound,

    #[error("Device changed — re-authentication required")]
    DeviceMismatch,

    #[error("Too many attempts — try again later")]
    RateLimited,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}