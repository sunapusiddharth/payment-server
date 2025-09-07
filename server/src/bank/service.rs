// src/bank/service.rs

use crate::bank::models::*;
use sqlx::PgPool;
use uuid::Uuid;
use tracing::info;

pub struct FakeBankService {
    db: PgPool,
}

impl FakeBankService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn link_account(&self, req: LinkBankAccountRequest) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO fake_bank_accounts (user_id, account_number, ifsc, name)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) DO UPDATE
            SET account_number = $2, ifsc = $3, name = $4
            "#,
            req.user_id,
            req.account_number,
            req.ifsc,
            req.name
        )
        .execute(&self.db)
        .await?;

        info!(user_id = %req.user_id, "Bank account linked");
        Ok(())
    }

    pub async fn get_balance(&self, user_id: Uuid) -> Result<BankBalanceResponse, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT account_number FROM fake_bank_accounts WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

        // Mask account number
        let masked = format!("XXXXXX{}", &row.account_number[row.account_number.len()-4..]);

        // Simulate low balance for testing
        let balance = if row.account_number == "LOW_BALANCE" {
            50000 // ₹500
        } else {
            5000000 // ₹50,000
        };

        Ok(BankBalanceResponse {
            account_number: masked,
            balance,
        })
    }

    pub async fn transfer(&self, from_account: &str, to_user_id: Uuid, amount: i64) -> Result<BankTransferResponse, sqlx::Error> {
        // Simulate UTR
        let utr = format!("UTR{}", Uuid::new_v4().to_string().replace("-", "").get(..10).unwrap());

        info!(from_account, to_user_id = %to_user_id, amount, utr, "Bank transfer simulated");

        Ok(BankTransferResponse {
            status: "success".to_string(),
            utr,
            message: "Transfer initiated".to_string(),
        })
    }
}