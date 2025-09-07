// src/fraud/rules.rs

use crate::fraud::models::FraudEvent;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct FraudRuleEngine {
    db: PgPool,
    // In-memory counters for velocity (reset every minute via background task)
    counters: Arc<Mutex<HashMap<String, VelocityCounter>>>,
        model: Option<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,

}

#[derive(Debug, Clone)]
struct VelocityCounter {
    count: u32,
    last_reset: chrono::DateTime<chrono::Utc>,
}

impl FraudRuleEngine {
    pub fn new(db: PgPool) -> Self {
        let model = std::fs::read("fraud_model.onnx")
            .ok()
            .and_then(|data| {
                let model = tract_onnx::onnx()
                    .model_for_read(&mut &data[..])
                    .ok()?
                    .into_optimized()
                    .ok()?
                    .into_runnable()
                    .ok()?;
                Some(model)
            });
        let engine = Self {
            db,
            counters: Arc::new(Mutex::new(HashMap::new())),
            model,
        };

        // Spawn background task to reset counters every minute
        let counters_clone = engine.counters.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                let mut counters = counters_clone.lock().await;
                let now = chrono::Utc::now();
                counters.retain(|_, counter| {
                    now - counter.last_reset < chrono::Duration::minutes(1)
                });
            }
        });

        engine
    }

    pub async fn compute_risk_score(&self, event: &FraudEvent) -> (i32, Vec<String>) {
        let mut score = 0;
        let mut reasons = Vec::new();

        // Rule 1: Velocity — >5 payments in 1 min from same user
        let user_key = format!("user:{}", event.from_user_id);
        let mut counters = self.counters.lock().await;
        let now = chrono::Utc::now();

        let entry = counters.entry(user_key.clone()).or_insert_with(|| VelocityCounter {
            count: 0,
            last_reset: now,
        });

        entry.count += 1;

        if entry.count > 5 {
            score += 40;
            reasons.push("High velocity: >5 payments in 1 minute".to_string());
        }

        // Rule 2: Amount > 90% of sender's balance
        if let Ok(balance) = self.get_user_balance(event.from_user_id).await {
            if event.amount as f64 > (balance as f64 * 0.9) {
                score += 30;
                reasons.push("Amount > 90% of wallet balance".to_string());
            }
        }

        // Rule 3: New device (if provided)
        if let Some(device_fp) = &event.device_fingerprint {
            if let Ok(is_known) = self.is_known_device(event.from_user_id, device_fp).await {
                if !is_known {
                    score += 20;
                    reasons.push("New device detected".to_string());
                }
            }
        }

        // Rule 4: Same receiver in short time
        if let Ok(same_receiver_count) = self.count_payments_to_receiver(event.from_user_id, event.to_user_id).await {
            if same_receiver_count > 3 {
                score += 20;
                reasons.push("Multiple payments to same receiver".to_string());
            }
        }

        // ML Model Inference (if available)
        if let Some(model) = &self.model {
            if let Ok(features) = self.extract_features(event).await {
                let input = tract_ndarray::ArrayD::from_shape_vec(
                    vec![1, features.len()],
                    features,
                ).unwrap();

                let result = model.run(tvec!(input.into())).unwrap();
                let anomaly_score = result[0].to_array_view::<f32>().unwrap().iter().next().unwrap();

                // Isolation Forest: -1 = anomaly, 1 = normal
                if *anomaly_score < 0.0 {
                    let ml_score = ((1.0 - anomaly_score.abs()) * 50.0) as i32; // 0 to 50
                    score += ml_score;
                    reasons.push("ML model detected anomaly".to_string());
                }
            }
        }

        (score, reasons)
    }

    async fn get_user_balance(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT balance FROM wallets WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.db)
        .await
    }

    async fn is_known_device(&self, user_id: Uuid, device_fp: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE id = $1 AND device_fingerprint = $2",
            user_id,
            device_fp
        )
        .fetch_one(&self.db)
        .await?;

        Ok(count.unwrap_or(0) > 0)
    }

    async fn count_payments_to_receiver(&self, from_user_id: Uuid, to_user_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM transaction_journal
            WHERE from_user_id = $1 AND to_user_id = $2
            AND created_at > NOW() - INTERVAL '5 minutes'
            "#,
            from_user_id,
            to_user_id
        )
        .fetch_one(&self.db)
        .await
    }


     async fn extract_features(&self, event: &FraudEvent) -> Result<Vec<f32>, sqlx::Error> {
        let balance = self.get_user_balance(event.from_user_id).await?;
        let time_since_last_tx = self.get_time_since_last_tx(event.from_user_id).await?;
        let tx_count_last_5min = self.count_payments_last_5min(event.from_user_id).await?;
        let avg_tx_amount_last_24h = self.get_avg_tx_amount_last_24h(event.from_user_id).await?;
        let is_new_device = if event.device_fingerprint.is_some() {
            self.is_known_device(event.from_user_id, event.device_fingerprint.as_ref().unwrap()).await? as i32
        } else { 0 };

        let hour_of_day = event.timestamp.hour() as f32;

        let features = vec![
            event.amount as f32 / 1_000_000.0, // normalize ₹ to lakhs
            time_since_last_tx as f32 / 3600.0, // hours
            tx_count_last_5min as f32,
            avg_tx_amount_last_24h as f32 / 1_000_000.0,
            is_new_device as f32,
            0.0, // is_new_ip — placeholder
            (event.amount as f32) / (balance.max(1) as f32), // balance_ratio
            hour_of_day / 24.0, // normalized
        ];

        Ok(features)
    }

    async fn get_time_since_last_tx(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let last_tx = sqlx::query_scalar!(
            "SELECT EXTRACT(EPOCH FROM (NOW() - MAX(created_at))) FROM transaction_journal WHERE from_user_id = $1",
            user_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(last_tx.unwrap_or(0) as i64)
    }

    async fn count_payments_last_5min(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM transaction_journal WHERE from_user_id = $1 AND created_at > NOW() - INTERVAL '5 minutes'",
            user_id
        )
        .fetch_one(&self.db)
        .await
    }

    async fn get_avg_tx_amount_last_24h(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT AVG(amount) FROM transaction_journal WHERE from_user_id = $1 AND created_at > NOW() - INTERVAL '24 hours'",
            user_id
        )
        .fetch_one(&self.db)
        .await
    }
}