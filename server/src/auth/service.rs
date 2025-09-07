// src/auth/service.rs

use crate::auth::models::*;
use crate::auth::crypto::*;
use sqlx::{PgPool, Executor};
use std::sync::Arc;
use tracing::{info, warn, instrument};
use std::time::Duration;

pub struct AuthService {
    db: PgPool,
    jwt_secret: String,
    otp_secret: String, // for HMAC mobile
    sms_client: Arc<dyn SmsClient>, // trait for SMS vendor
}

#[async_trait::async_trait]
pub trait SmsClient: Send + Sync {
    async fn send_otp(&self, mobile: &str, otp: &str) -> Result<(), Box<dyn std::error::Error>>;
}

impl AuthService {
    pub fn new(db: PgPool, jwt_secret: String, otp_secret: String, sms_client: Arc<dyn SmsClient>) -> Self {
        Self {
            db,
            jwt_secret,
            otp_secret,
            sms_client,
        }
    }

    #[instrument(skip(self), fields(mobile = %req.mobile))]
    pub async fn register(&self, req: RegisterRequest) -> Result<(), AuthError> {
        req.validate()?;

        let mobile_hash = hash_mobile(&req.mobile, &self.otp_secret);

        // Check if user already exists
        if sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE mobile_hash = $1)",
            &mobile_hash
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false) {
            return Ok(()); // idempotent — don't error
        }

        // Generate & store OTP
        let otp = generate_otp();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

        sqlx::query!(
            r#"
            INSERT INTO otp_store (mobile_hash, otp, expires_at, attempts)
            VALUES ($1, $2, $3, 0)
            ON CONFLICT (mobile_hash) DO UPDATE
            SET otp = $2, expires_at = $3, attempts = 0
            "#,
            mobile_hash,
            otp,
            expires_at
        )
        .execute(&self.db)
        .await?;

        // Send via SMS — fire and forget (don't block on failure)
        let sms_mobile = req.mobile.clone();
        let sms_otp = otp.clone();
        let sms_client = self.sms_client.clone();
        tokio::spawn(async move {
            if let Err(e) = sms_client.send_otp(&sms_mobile, &sms_otp).await {
                warn!(mobile = %sms_mobile, error = %e, "Failed to send OTP");
            }
        });

        info!(mobile = %req.mobile, "OTP sent for registration");
        Ok(())
    }

    #[instrument(skip(self), fields(mobile = %req.mobile))]
    pub async fn verify_otp(&self, req: VerifyOtpRequest) -> Result<LoginResponse, AuthError> {
        req.validate()?;

        let mobile_hash = hash_mobile(&req.mobile, &self.otp_secret);

        // Fetch OTP record
        let record = sqlx::query!(
            "SELECT otp, expires_at, attempts FROM otp_store WHERE mobile_hash = $1",
            &mobile_hash
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AuthError::OtpExpired)?;

        // Check expiry
        if chrono::Utc::now() > record.expires_at {
            return Err(AuthError::OtpExpired);
        }

        // Check attempts
        if record.attempts >= 3 {
            return Err(AuthError::RateLimited);
        }

        // Validate OTP
        if record.otp != req.otp {
            // Increment attempts
            sqlx::query!(
                "UPDATE otp_store SET attempts = attempts + 1 WHERE mobile_hash = $1",
                &mobile_hash
            )
            .execute(&self.db)
            .await?;
            return Err(AuthError::InvalidOtp);
        }

        // OTP correct — delete it
        sqlx::query!("DELETE FROM otp_store WHERE mobile_hash = $1", &mobile_hash)
            .execute(&self.db)
            .await?;

        // Get or create user
        let user_id = self.get_or_create_user(&mobile_hash, req.device_fingerprint.as_deref()).await?;

        // Issue JWT
        let access_token = create_jwt(
            &user_id,
            req.device_fingerprint.as_deref(),
            &self.jwt_secret,
            900, // 15 min
        )?;

        let refresh_token = self.create_refresh_token(&user_id, req.device_fingerprint.as_deref()).await?;

        info!(user_id = %user_id, "User logged in");
        Ok(LoginResponse {
            access_token,
            refresh_token,
            expires_in: 900,
            user_id,
        })
    }

    async fn get_or_create_user(
        &self,
        mobile_hash: &str,
        device_fingerprint: Option<&str>,
    ) -> Result<Uuid, AuthError> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, mobile_hash, device_fingerprint, created_at, updated_at FROM users WHERE mobile_hash = $1",
            mobile_hash
        )
        .fetch_optional(&self.db)
        .await?;

        match user {
            Some(u) => {
                // If device changed — flag for step-up auth later (optional)
                if let Some(fp) = device_fingerprint {
                    if u.device_fingerprint.as_deref() != Some(fp) {
                        warn!(user_id = %u.id, "Device changed — consider step-up auth");
                        // Optional: update device_fingerprint
                        sqlx::query!(
                            "UPDATE users SET device_fingerprint = $1 WHERE id = $2",
                            fp,
                            u.id
                        )
                        .execute(&self.db)
                        .await?;
                    }
                }
                Ok(u.id)
            }
            None => {
                let user_id = Uuid::new_v4();
                sqlx::query!(
                    r#"
                    INSERT INTO users (id, mobile_hash, device_fingerprint)
                    VALUES ($1, $2, $3)
                    "#,
                    user_id,
                    mobile_hash,
                    device_fingerprint
                )
                .execute(&self.db)
                .await?;
                Ok(user_id)
            }
        }
    }

    async fn create_refresh_token(
        &self,
        user_id: &Uuid,
        device_fingerprint: Option<&str>,
    ) -> Result<String, AuthError> {
        let token = generate_refresh_token();
        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        sqlx::query!(
            r#"
            INSERT INTO refresh_tokens (token, user_id, expires_at, device_fingerprint)
            VALUES ($1, $2, $3, $4)
            "#,
            &token,
            user_id,
            expires_at,
            device_fingerprint
        )
        .execute(&self.db)
        .await?;

        Ok(token)
    }

    #[instrument(skip(self))]
    pub async fn refresh_token(&self, token: &str) -> Result<LoginResponse, AuthError> {
        let record = sqlx::query_as!(
            RefreshToken,
            "SELECT token, user_id, expires_at, revoked, device_fingerprint FROM refresh_tokens WHERE token = $1",
            token
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AuthError::InvalidOtp)?; // reuse error for simplicity

        if record.revoked {
            return Err(AuthError::InvalidOtp);
        }

        if chrono::Utc::now() > record.expires_at {
            return Err(AuthError::OtpExpired);
        }

        // Issue new tokens
        let access_token = create_jwt(
            &record.user_id,
            record.device_fingerprint.as_deref(),
            &self.jwt_secret,
            900,
        )?;

        let new_refresh_token = self.create_refresh_token(&record.user_id, record.device_fingerprint.as_deref()).await?;

        // Revoke old token
        sqlx::query!("UPDATE refresh_tokens SET revoked = TRUE WHERE token = $1", token)
            .execute(&self.db)
            .await?;

        info!(user_id = %record.user_id, "Token refreshed");
        Ok(LoginResponse {
            access_token,
            refresh_token: new_refresh_token,
            expires_in: 900,
            user_id: record.user_id,
        })
    }

    #[instrument(skip(self))]
    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        sqlx::query!("UPDATE refresh_tokens SET revoked = TRUE WHERE token = $1", token)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

fn generate_refresh_token() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| rng.gen_range(0..16).to_string(16))
        .collect()
}