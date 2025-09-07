// tests/unit/auth.rs
use crate::common::{TestContext, new_uuid, generate_jwt};
use payment_system::auth::{AuthService, models::*, crypto::*};
use mockall::mock;
use async_trait::async_trait;

mock! {
    pub SmsClient {}
    #[async_trait]
    impl SmsClient for SmsClient {
        async fn send_otp(&self, mobile: &str, otp: &str) -> Result<(), Box<dyn std::error::Error>>;
    }
}

#[tokio::test]
async fn test_register_sends_otp() {
    let ctx = TestContext::new().await;
    let mut mock_sms = MockSmsClient::new();
    mock_sms.expect_send_otp()
        .returning(|_, _| Ok(()));

    let service = AuthService::new(
        ctx.db.clone(),
        "jwt_secret".to_string(),
        "otp_secret".to_string(),
        Arc::new(mock_sms),
    );

    let req = RegisterRequest {
        mobile: "+919876543210".to_string(),
    };

    service.register(req).await.unwrap();

    // Verify OTP stored in DB
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM otp_store WHERE mobile_hash = $1",
        hash_mobile("+919876543210", "otp_secret")
    )
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_verify_otp_creates_user() {
    let ctx = TestContext::new().await;
    let service = AuthService::new(
        ctx.db.clone(),
        "jwt_secret".to_string(),
        "otp_secret".to_string(),
        Arc::new(MockSmsClient::new()),
    );

    // Pre-store OTP
    let mobile_hash = hash_mobile("+919876543210", "otp_secret");
    let otp = "123456";
    sqlx::query!(
        "INSERT INTO otp_store (mobile_hash, otp, expires_at) VALUES ($1, $2, NOW() + INTERVAL '5 minutes')",
        mobile_hash,
        otp
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    let req = VerifyOtpRequest {
        mobile: "+919876543210".to_string(),
        otp: otp.to_string(),
        device_fingerprint: None,
    };

    let resp = service.verify_otp(req).await.unwrap();

    assert!(!resp.access_token.is_empty());
    assert!(!resp.refresh_token.is_empty());

    // Verify user created
    let user_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE mobile_hash = $1",
        mobile_hash
    )
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    assert_eq!(user_count, 1);
}