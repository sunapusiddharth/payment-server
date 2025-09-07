// src/bank/handlers.rs

use axum::{
    Extension,
    Json,
    extract::Path,
    http::StatusCode,
};
use crate::bank::{service::FakeBankService, models::*};

pub async fn link_account(
    Extension(bank_service): Extension<std::sync::Arc<FakeBankService>>,
    Json(payload): Json<LinkBankAccountRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    bank_service.link_account(payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(serde_json::json!({
        "status": "linked",
        "message": "Account linked successfully"
    })))
}

pub async fn get_balance(
    Path(user_id): Path<Uuid>,
    Extension(bank_service): Extension<std::sync::Arc<FakeBankService>>,
) -> Result<Json<BankBalanceResponse>, (StatusCode, Json<serde_json::Value>)> {
    let resp = bank_service.get_balance(user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Account not linked" })),
            )
        })?;

    Ok(Json(resp))
}

pub async fn transfer(
    Extension(bank_service): Extension<std::sync::Arc<FakeBankService>>,
    Json(payload): Json<serde_json::Value>, // for simplicity
) -> Result<Json<BankTransferResponse>, (StatusCode, Json<serde_json::Value>)> {
    let from_account = payload["from_account"].as_str().unwrap_or("1234567890");
    let to_user_id = Uuid::parse_str(payload["to_user_id"].as_str().unwrap()).map_err(|_| (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Invalid to_user_id" })),
    ))?;
    let amount = payload["amount"].as_i64().unwrap_or(10000);

    let resp = bank_service.transfer(from_account, to_user_id, amount)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(resp))
}