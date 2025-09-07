// src/bank/models.rs

use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Debug, Deserialize)]
pub struct LinkBankAccountRequest {
    pub user_id: Uuid,
    pub account_number: String,
    pub ifsc: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct BankBalanceResponse {
    pub account_number: String,
    pub balance: i64,
}

#[derive(Debug, Serialize)]
pub struct BankTransferResponse {
    pub status: String,
    pub utr: String,
    pub message: String,
}