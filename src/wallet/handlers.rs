// src/wallet/handlers.rs

use axum::{
    Extension,
    extract::Path,
    Json,
};
use uuid::Uuid;
use crate::wallet::{WalletService, WalletError};

pub async fn get_balance(
    Extension(wallet_service): Extension<std::sync::Arc<WalletService>>,
    user_id: Uuid, // from JWT middleware
) -> Result<Json<i64>, (http::StatusCode, Json<serde_json::Value>)> {
    let balance = wallet_service.get_balance_cached(&user_id)
        .await
        .map_err(|e| {
            (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(balance))
}