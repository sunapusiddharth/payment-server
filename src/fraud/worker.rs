// src/fraud/worker.rs

use nats::asynk::Connection;
use crate::fraud::models::{FraudEvent, FraudFlag, FraudError};
use crate::fraud::rules::FraudRuleEngine;
use sqlx::PgPool;
use tracing::{info, error, warn};
use reqwest;
use std::sync::Arc;

pub struct FraudWorker {
    nats: Connection,
    rule_engine: FraudRuleEngine,
    db: PgPool,
    alert_webhook_url: String,
    client: reqwest::Client,
}

impl FraudWorker {
    pub async fn new(
        nats_url: &str,
        db: PgPool,
        alert_webhook_url: String,
    ) -> Result<Self, FraudError> {
        let nats = nats::asynk::connect(nats_url).await?;
        let rule_engine = FraudRuleEngine::new(db.clone());

        Ok(Self {
            nats,
            rule_engine,
            db,
            alert_webhook_url,
            client: reqwest::Client::new(),
        })
    }

    pub async fn start(&self) -> Result<(), FraudError> {
        let sub = self.nats.subscribe("fraud.payment.created").await?;

        info!("Fraud worker started, listening on 'fraud.payment.created'");

        while let Some(msg) = sub.next().await {
            let payload = match std::str::from_utf8(&msg.data) {
                Ok(s) => s,
                Err(e) => {
                    error!(error = %e, "Invalid UTF-8 in NATS message");
                    continue;
                }
            };

            let event: FraudEvent = match serde_json::from_str(payload) {
                Ok(e) => e,
                Err(e) => {
                    error!(error = %e, "Failed to parse FraudEvent: {}", payload);
                    continue;
                }
            };

            tokio::spawn(self.handle_event(event));
        }

        Ok(())
    }

    async fn handle_event(&self, event: FraudEvent) {
        let (risk_score, reasons) = self.rule_engine.compute_risk_score(&event).await;

        metrics::gauge!("fraud_risk_score", risk_score as f64);

        if risk_score > 70 {
            warn!(
                tx_id = %event.tx_id,
                risk_score = risk_score,
                reasons = ?reasons,
                "High risk transaction flagged"
            );

            // Record in DB
            if let Err(e) = self.flag_transaction(&event, risk_score, &reasons.join("; ")).await {
                error!(error = %e, "Failed to flag transaction");
            }

            // Send alert
            if let Err(e) = self.send_alert(&event, risk_score, &reasons).await {
                error!(error = %e, "Failed to send alert");
            }

            metrics::counter!("fraud_alerts_triggered", 1);
            metrics::histogram!("fraud_score_distribution", risk_score as f64);
        }
    }

    async fn flag_transaction(
        &self,
        event: &FraudEvent,
        risk_score: i32,
        reason: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO fraud_flags (tx_id, risk_score, reason)
            VALUES ($1, $2, $3)
            ON CONFLICT (tx_id) DO NOTHING
            "#,
            event.tx_id,
            risk_score,
            reason
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn send_alert(
        &self,
        event: &FraudEvent,
        risk_score: i32,
        reasons: &[String],
    ) -> Result<(), reqwest::Error> {
        #[derive(Serialize)]
        struct AlertPayload {
            tx_id: String,
            from_user_id: String,
            to_user_id: String,
            amount: u64,
            risk_score: i32,
            reasons: Vec<String>,
            timestamp: String,
        }

        let payload = AlertPayload {
            tx_id: event.tx_id.to_string(),
            from_user_id: event.from_user_id.to_string(),
            to_user_id: event.to_user_id.to_string(),
            amount: event.amount,
            risk_score,
            reasons: reasons.to_vec(),
            timestamp: event.timestamp.to_rfc3339(),
        };

        self.client
            .post(&self.alert_webhook_url)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }
}