// src/wallet_service.rs

use crate::models::{Wallet, CreditDebitRequest, WalletError, CreateWalletRequest};
use sqlx::{PgPool, Executor};
use std::sync::Arc;
use tracing::{info, error, instrument};
use opentelemetry::trace::TraceContextExt;
use metrics::{counter, histogram};

pub struct WalletService {
    db: PgPool,
    // Optional: Redis client for caching
}

impl WalletService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    #[instrument(skip(self), fields(user_id = %req.user_id))]
    pub async fn create_wallet(&self, req: CreateWalletRequest) -> Result<Wallet, WalletError> {
        req.validate()?;

        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            INSERT INTO wallets (user_id, balance, version)
            VALUES ($1, 0, 0)
            ON CONFLICT (user_id) DO NOTHING
            RETURNING user_id, balance, version, created_at, updated_at
            "#,
            req.user_id
        )
        .fetch_one(&self.db)
        .await?;

        info!("Wallet created for user {}", req.user_id);
        Ok(wallet)
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_balance(&self, user_id: &Uuid) -> Result<i64, WalletError> {
        let balance = sqlx::query_scalar!(
            "SELECT balance FROM wallets WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| WalletError::WalletNotFound(*user_id))?;

        Ok(balance)
    }

    #[instrument(skip(self), fields(user_id = %req.user_id, amount = req.amount))]
pub async fn credit(&self, req: &CreditDebitRequest) -> Result<Wallet, WalletError> {
    let start = std::time::Instant::now();
    let result = self.process_transaction(req, true).await;

    let status = match &result {
        Ok(_) => "success",
        Err(WalletError::InsufficientBalance) => "insufficient_balance",
        Err(WalletError::ConcurrencyConflict) => "concurrency_conflict",
        Err(_) => "error",
    };

    histogram!("wallet_credit_duration_seconds", start.elapsed().as_secs_f64());
    counter!("wallet_credit_total", 1, "status" => status);

    result
}

    #[instrument(skip(self), fields(user_id = %req.user_id, amount = req.amount))]
    pub async fn debit(&self, req: &CreditDebitRequest) -> Result<Wallet, WalletError> {
        self.process_transaction(req, false).await
    }

    async fn process_transaction(
        &self,
        req: &CreditDebitRequest,
        is_credit: bool,
    ) -> Result<Wallet, WalletError> {
        req.validate()?;

        // Step 1: Check idempotency
        if self.is_idempotent(&req.idempotency_key, &req.user_id).await? {
            return Err(WalletError::DuplicateIdempotencyKey);
        }

        // Step 2: Begin serializable transaction
        let mut tx = self.db.begin().await?;

        // Step 3: Lock wallet row + get current state
        let mut wallet = sqlx::query_as!(
            Wallet,
            r#"
            SELECT user_id, balance, version, created_at, updated_at
            FROM wallets
            WHERE user_id = $1
            FOR UPDATE
            "#,
            req.user_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| WalletError::WalletNotFound(req.user_id))?;

        // Step 4: Validate balance for debit
        if !is_credit && wallet.balance < req.amount as i64 {
            return Err(WalletError::InsufficientBalance);
        }

        // Step 5: Calculate new balance
        let amount_i64 = req.amount as i64;
        let new_balance = if is_credit {
            wallet.balance + amount_i64
        } else {
            wallet.balance - amount_i64
        };

        // Step 6: Update with version check (OCC)
        let new_version = wallet.version + 1;
        let rows_affected = sqlx::query!(
            r#"
            UPDATE wallets
            SET balance = $1, version = $2, updated_at = NOW()
            WHERE user_id = $3 AND version = $4
            "#,
            new_balance,
            new_version,
            req.user_id,
            wallet.version
        )
        .execute(&mut *tx)
        .await?;

        if rows_affected.rows_affected() == 0 {
            return Err(WalletError::ConcurrencyConflict);
        }

        // Step 7: Record idempotency
        sqlx::query!(
            r#"
            INSERT INTO idempotency_keys (idempotency_key, user_id)
            VALUES ($1, $2)
            "#,
            &req.idempotency_key,
            req.user_id
        )
        .execute(&mut *tx)
        .await?;

        // Step 8: Commit
        tx.commit().await?;

        // Step 9: Invalidate cache (if using Redis) — async fire-and-forget
        // self.invalidate_cache(req.user_id).await;

        // Step 10: Return updated wallet
        wallet.balance = new_balance;
        wallet.version = new_version;
        wallet.updated_at = chrono::Utc::now();

        info!(
            user_id = %req.user_id,
            amount = req.amount,
            operation = if is_credit { "credit" } else { "debit" },
            "Wallet updated"
        );

        Ok(wallet)
    }

    async fn is_idempotent(&self, key: &str, user_id: &Uuid) -> Result<bool, WalletError> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM idempotency_keys WHERE idempotency_key = $1 AND user_id = $2)",
            key,
            user_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(exists.unwrap_or(false))
    }


    pub async fn get_balance_cached(&self, user_id: &Uuid) -> Result<i64, WalletError> {
        let key = format!("wallet:balance:{}", user_id);
        let mut conn = self.redis_client.get_async_connection().await?;

        if let Ok(cached) = conn.get::<_, Option<i64>>(&key).await {
            if let Some(balance) = cached {
                return Ok(balance);
            }
        }

        // Cache miss → fetch from DB
        let balance = self.get_balance(user_id).await?;
        
        // Write-through cache
        let _: () = conn.set_ex(&key, balance, 300).await?; // 5 min TTL

        Ok(balance)
    }

    // Call this after every credit/debit
    pub async fn invalidate_cache(&self, user_id: Uuid) {
        let key = format!("wallet:balance:{}", user_id);
        let _ = self.redis_client.get_async_connection().await
            .and_then(|mut conn| async move {
                let _: () = conn.del(key).await.unwrap_or(());
                Ok::<(), redis::RedisError>(())
            }).await;
    }


    pub async fn top_up_from_bank(&self, user_id: Uuid, amount: u64) -> Result<Wallet, WalletError> {
    let client = reqwest::Client::new();
    let transfer_req = serde_json::json!({
        "from_account": "1234567890",
        "to_user_id": user_id,
        "amount": amount
    });

    let _: BankTransferResponse = client
        .post("http://localhost:3002/bank/transfer")
        .json(&transfer_req)
        .send()
        .await?
        .json()
        .await?;

    // Credit wallet
    let req = CreditDebitRequest {
        user_id,
        amount,
        idempotency_key: Uuid::new_v4().to_string(),
    };
    self.credit(&req).await
}
}