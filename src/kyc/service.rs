// src/kyc/service.rs

use crate::kyc::models::*;
use sqlx::PgPool;
use tracing::info;

pub struct FakeKycService {
    db: PgPool,
}

impl FakeKycService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn verify_kyc(&self, req: KycVerifyRequest) -> Result<KycVerifyResponse, sqlx::Error> {
        // Force reject for testing
        if let Some(ref pan) = req.pan {
            if pan == "REJECT_ME" {
                sqlx::query!(
                    r#"
                    INSERT INTO fake_kyc_verifications 
                    (user_id, pan, name, dob, status, reason)
                    VALUES ($1, $2, $3, $4, 'rejected', 'Test rejection')
                    ON CONFLICT (user_id) DO UPDATE
                    SET status = 'rejected', reason = 'Test rejection'
                    "#,
                    req.user_id,
                    pan,
                    req.name,
                    req.dob
                )
                .execute(&self.db)
                .await?;

                return Ok(KycVerifyResponse {
                    status: "rejected".to_string(),
                    kyc_tier: "basic".to_string(),
                    message: "KYC rejected for testing".to_string(),
                });
            }
        }

        // Approve by default
        let status = "approved";
        let kyc_tier = "full";

        sqlx::query!(
            r#"
            INSERT INTO fake_kyc_verifications 
            (user_id, pan, aadhaar, name, dob, status, verified_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (user_id) DO UPDATE
            SET pan = $2, aadhaar = $3, name = $4, dob = $5, status = $6, verified_at = NOW()
            "#,
            req.user_id,
            req.pan,
            req.aadhaar,
            req.name,
            req.dob,
            status
        )
        .execute(&self.db)
        .await?;

        info!(user_id = %req.user_id, "KYC approved");
        Ok(KycVerifyResponse {
            status: status.to_string(),
            kyc_tier: kyc_tier.to_string(),
            message: "KYC verified successfully".to_string(),
        })
    }

    pub async fn get_kyc_status(&self, user_id: Uuid) -> Result<Option<KycVerification>, sqlx::Error> {
        sqlx::query_as!(
            KycVerification,
            "SELECT user_id, status, reason FROM fake_kyc_verifications WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await
    }
}