// src/auth/handlers.rs

use axum::{
    Extension,
    Json,
    http::StatusCode,
};
use serde_json::json;
use crate::auth::{AuthService, models::*};

pub async fn register(
    Extension(auth_service): Extension<std::sync::Arc<AuthService>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    auth_service.register(payload)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(json!({ "message": "OTP sent" })))
}

pub async fn verify_otp(
    Extension(auth_service): Extension<std::sync::Arc<AuthService>>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    // let resp = auth_service.verify_otp(payload)
    //     .await
    //     .map_err(|e| {
    //         (
    //             StatusCode::UNAUTHORIZED,
    //             Json(json!({ "error": e.to_string() })),
    //         )
    //     })?;

    // Ok(Json(resp))
    // In AuthService::verify_otp
if let Some(kyc_data) = &req.kyc_data { // add to VerifyOtpRequest
    let client = reqwest::Client::new();
    let kyc_resp: KycVerifyResponse = client
        .post("http://localhost:3001/kyc/verify")
        .json(&kyc_data)
        .send()
        .await?
        .json()
        .await?;

    if kyc_resp.status == "approved" {
        // Update user’s KYC tier in daily_limits table
        sqlx::query!(
            "INSERT INTO daily_limits (user_id, kyc_tier) VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET kyc_tier = $2",
            user_id,
            "full"
        )
        .execute(&self.db)
        .await?;
    }
}
}