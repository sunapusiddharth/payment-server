// src/payment/service.rs

use crate::payment::models::*;
use crate::wallet::WalletService;
use sqlx::{PgPool, Executor};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

pub struct PaymentService {
    db: PgPool,
    wallet_service: Arc<WalletService>,
    otp_secret: String, // for hashing mobile
    nats_client: Arc<dyn NatsClient>, // for fraud events
}

#[async_trait::async_trait]
pub trait NatsClient: Send + Sync {
    async fn publish_fraud_event(&self, event: &FraudEvent) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Debug, Serialize)]
pub struct FraudEvent {
    pub tx_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount: u64,
    pub device_fingerprint: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PaymentService {
    pub fn new(
        db: PgPool,
        wallet_service: Arc<WalletService>,
        otp_secret: String,
        nats_client: Arc<dyn NatsClient>,
    ) -> Self {
        Self {
            db,
            wallet_service,
            otp_secret,
            nats_client,
        }
    }

    #[instrument(skip(self), fields(from_user_id = %from_user_id, amount = req.amount))]
    pub async fn pay_by_phone(
        &self,
        from_user_id: Uuid,
        req: PayByPhoneRequest,
    ) -> Result<PaymentResponse, PaymentError> {
        let start = std::time::Instant::now();
        req.validate()?;

        // Step 1: Check idempotency
        if self.is_idempotent(&req.idempotency_key).await? {
            return Err(PaymentError::DuplicateIdempotencyKey);
        }

        // Step 2: Resolve to_mobile → to_user_id
        let to_user_id = self.resolve_mobile(&req.to_mobile).await?;

        // Step 3: Validate same user
        if from_user_id == to_user_id {
            return Err(PaymentError::UserNotFound("Cannot send to self".to_string()));
        }

        // Step 4: Check daily limit
        self.check_daily_limit(from_user_id, req.amount).await?;

        // Step 5: Begin payment — use DB transaction for atomicity
        let mut tx = self.db.begin().await?;

        // Step 6: Debit sender
        let debit_req = crate::wallet::CreditDebitRequest {
            user_id: from_user_id,
            amount: req.amount,
            idempotency_key: format!("debit_{}", req.idempotency_key),
        };
        self.wallet_service.debit(&debit_req).await?;

        // Step 7: Credit receiver
        let credit_req = crate::wallet::CreditDebitRequest {
            user_id: to_user_id,
            amount: req.amount,
            idempotency_key: format!("credit_{}", req.idempotency_key),
        };
        self.wallet_service.credit(&credit_req).await?;

        // Step 8: Record in journal
        let tx_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO transaction_journal (tx_id, from_user_id, to_user_id, amount, status, idempotency_key)
            VALUES ($1, $2, $3, $4, 'SUCCESS', $5)
            "#,
            tx_id,
            from_user_id,
            to_user_id,
            req.amount as i64,
            &req.idempotency_key
        )
        .execute(&mut *tx)
        .await?;

        // Step 9: Update daily limit
        self.update_daily_limit(&mut tx, from_user_id, req.amount).await?;

        // Step 10: Commit
        tx.commit().await?;

        // Step 11: Emit fraud event (async, fire-and-forget)
        let event = FraudEvent {
            tx_id,
            from_user_id,
            to_user_id,
            amount: req.amount,
            device_fingerprint: None, // get from request context
            ip_address: None,         // get from request context
            timestamp: chrono::Utc::now(),
        };
        let nc = self.nats_client.clone();
        tokio::spawn(async move {
            if let Err(e) = nc.publish_fraud_event(&event).await {
                tracing::error!(error = %e, "Failed to publish fraud event");
            }
        });

        info!(tx_id = %tx_id, "Payment completed");
         let status = match &result {
        Ok(_) => "success",
        Err(PaymentError::InsufficientBalance) => "insufficient_balance",
        Err(PaymentError::DailyLimitExceeded) => "daily_limit_exceeded",
        Err(_) => "error",
    };

    histogram!("payment_duration_seconds", start.elapsed().as_secs_f64());
    counter!("payment_total", 1, "status" => status, "method" => "phone");
        Ok(PaymentResponse {
            tx_id,
            from_user_id,
            to_user_id,
            amount: req.amount,
            status: PaymentStatus::Success,
            timestamp: chrono::Utc::now(),
        })
    }

    #[instrument(skip(self), fields(from_user_id = %from_user_id, amount = req.amount))]
    pub async fn pay_by_qr(
        &self,
        from_user_id: Uuid,
        req: PayByQrRequest,
    ) -> Result<PaymentResponse, PaymentError> {
        req.validate()?;

        // Decode QR: "payment://user/<uuid>"
        let to_user_id = self.decode_qr(&req.qr_code)?;

        // Same logic as pay_by_phone — reuse
        let phone_req = PayByPhoneRequest {
            to_mobile: "QR_PAYMENT".to_string(), // dummy — we have user_id
            amount: req.amount,
            idempotency_key: req.idempotency_key,
        };

        // Override resolved user_id
        let mut tx = self.db.begin().await?;
        if self.is_idempotent(&req.idempotency_key).await? {
            return Err(PaymentError::DuplicateIdempotencyKey);
        }
        self.check_daily_limit(from_user_id, req.amount).await?;
        let debit_req = crate::wallet::CreditDebitRequest {
            user_id: from_user_id,
            amount: req.amount,
            idempotency_key: format!("debit_{}", req.idempotency_key),
        };
        self.wallet_service.debit(&debit_req).await?;
        let credit_req = crate::wallet::CreditDebitRequest {
            user_id: to_user_id,
            amount: req.amount,
            idempotency_key: format!("credit_{}", req.idempotency_key),
        };
        self.wallet_service.credit(&credit_req).await?;
        let tx_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO transaction_journal (tx_id, from_user_id, to_user_id, amount, status, idempotency_key)
            VALUES ($1, $2, $3, $4, 'SUCCESS', $5)
            "#,
            tx_id,
            from_user_id,
            to_user_id,
            req.amount as i64,
            &req.idempotency_key
        )
        .execute(&mut *tx)
        .await?;
        self.update_daily_limit(&mut tx, from_user_id, req.amount).await?;
        tx.commit().await?;

        let event = FraudEvent {
            tx_id,
            from_user_id,
            to_user_id,
            amount: req.amount,
            device_fingerprint: None,
            ip_address: None,
            timestamp: chrono::Utc::now(),
        };
        let nc = self.nats_client.clone();
        tokio::spawn(async move {
            if let Err(e) = nc.publish_fraud_event(&event).await {
                tracing::error!(error = %e, "Failed to publish fraud event");
            }
        });

        Ok(PaymentResponse {
            tx_id,
            from_user_id,
            to_user_id,
            amount: req.amount,
            status: PaymentStatus::Success,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn resolve_mobile(&self, mobile: &str) -> Result<Uuid, PaymentError> {
        let mobile_hash = hash_mobile(mobile, &self.otp_secret);
        let user_id = sqlx::query_scalar!(
            "SELECT id FROM users WHERE mobile_hash = $1",
            &mobile_hash
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| PaymentError::UserNotFound(mobile.to_string()))?;

        Ok(user_id)
    }

    fn decode_qr(&self, qr: &str) -> Result<Uuid, PaymentError> {
        // Format: payment://user/<uuid>
        let parts: Vec<&str> = qr.split('/').collect();
        if parts.len() < 4 || parts[2] != "user" {
            return Err(PaymentError::InvalidQrCode);
        }
        Uuid::from_str(parts[3])
            .map_err(|_| PaymentError::InvalidQrCode)
    }

    async fn check_daily_limit(&self, user_id: Uuid, amount: u64) -> Result<(), PaymentError> {
        let today = chrono::Utc::now().date_naive();
        let row = sqlx::query!(
            r#"
            SELECT amount_used, reset_date, kyc_tier
            FROM daily_limits
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?;

        let (amount_used, reset_date, kyc_tier) = match row {
            Some(r) => (r.amount_used.unwrap_or(0), r.reset_date, r.kyc_tier.unwrap_or("basic".to_string())),
            None => (0, today, "basic".to_string()),
        };

        // Reset if new day
        if reset_date < today {
            sqlx::query!(
                "UPDATE daily_limits SET amount_used = 0, reset_date = $1 WHERE user_id = $2",
                today,
                user_id
            )
            .execute(&self.db)
            .await?;
            return self.check_daily_limit(user_id, amount).await; // recurse
        }

        let limit = if kyc_tier == "full" { 100_000_00 } else { 10_000_00 }; // in paise
        if (amount_used + amount as i64) > limit {
            return Err(PaymentError::DailyLimitExceeded);
        }

        Ok(())
    }

    async fn update_daily_limit(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        amount: u64,
    ) -> Result<(), PaymentError> {
        let today = chrono::Utc::now().date_naive();
        let rows = sqlx::query!(
            r#"
            INSERT INTO daily_limits (user_id, amount_used, reset_date, kyc_tier)
            VALUES ($1, $2, $3, 'basic')
            ON CONFLICT (user_id) DO UPDATE
            SET amount_used = daily_limits.amount_used + $2
            WHERE daily_limits.user_id = $1
            "#,
            user_id,
            amount as i64,
            today
        )
        .execute(&mut *tx)
        .await?;

        Ok(())
    }

    async fn is_idempotent(&self, key: &str) -> Result<bool, PaymentError> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM transaction_journal WHERE idempotency_key = $1)",
            key
        )
        .fetch_one(&self.db)
        .await?;

        Ok(exists.unwrap_or(false))
    }
}