// src/transaction/models.rs

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TransactionItem {
    pub tx_id: Uuid,
    pub amount: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub counterparty_mobile: Option<String>, // masked
    pub transaction_type: String, // "sent" or "received"
}