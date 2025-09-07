// src/kyc/handlers.rs

use axum::{
    Extension,
    Json,
    http::StatusCode,
};
use crate::kyc::{service::FakeKycService, models::*};

pub async fn verify_kyc(
    Extension(kyc_service): Extension<std::sync::Arc<FakeKycService>>,
    Json(payload): Json<KycVerifyRequest>,
) -> Result<Json<KycVerifyResponse>, (StatusCode, Json<serde_json::Value>)> {
    let resp = kyc_service.verify_kyc(payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(resp))
}