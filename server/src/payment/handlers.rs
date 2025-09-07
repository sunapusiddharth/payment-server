// src/payment/handlers.rs

use axum::{
    Extension,
    Json,
    extract::Extension as Ext,
};
use uuid::Uuid;
use crate::payment::{PaymentService, models::*};

pub async fn pay_by_phone(
    Extension(payment_service): Extension<std::sync::Arc<PaymentService>>,
    user_id: Uuid, // from JWT middleware
    Json(payload): Json<PayByPhoneRequest>,
) -> Result<Json<PaymentResponse>, (http::StatusCode, Json<serde_json::Value>)> {
    let resp = payment_service.pay_by_phone(user_id, payload)
        .await
        .map_err(|e| {
            (
                http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(resp))
}