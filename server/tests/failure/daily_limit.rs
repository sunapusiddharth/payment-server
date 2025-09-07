// tests/failure/daily_limit.rs
use crate::common::{TestContext, new_uuid};

#[tokio::test]
async fn test_daily_limit_blocks_payments() {
    let ctx = TestContext::new().await;
    let wallet_service = Arc::new(payment_system::wallet::WalletService::new(ctx.db.clone()));
    let service = PaymentService::new(
        ctx.db.clone(),
        wallet_service.clone(),
        "otp_secret".to_string(),
        Arc::new(MockNatsClient::new()),
    );

    let user_id = new_uuid();
    wallet_service.create_wallet(CreateWalletRequest { user_id }).await.unwrap();
    wallet_service.credit(&CreditDebitRequest {
        user_id,
        amount: 5000000, // ₹50,000
        idempotency_key: "credit_1".to_string(),
    }).await.unwrap();

    // Set user to basic KYC tier (₹10,000 daily limit)
    sqlx::query!(
        "INSERT INTO daily_limits (user_id, kyc_tier) VALUES ($1, 'basic')",
        user_id
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // Create receiver
    let receiver_id = new_uuid();
    let mobile_hash = payment_system::auth::crypto::hash_mobile("+919876543210", "otp_secret");
    sqlx::query!(
        "INSERT INTO users (id, mobile_hash) VALUES ($1, $2)",
        receiver_id,
        mobile_hash
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // First payment: ₹8,000 (within limit)
    let req1 = PayByPhoneRequest {
        to_mobile: "+919876543210".to_string(),
        amount: 800000,
        idempotency_key: "payment_1".to_string(),
    };
    service.pay_by_phone(user_id, req1).await.unwrap();

    // Second payment: ₹3,000 (exceeds limit)
    let req2 = PayByPhoneRequest {
        to_mobile: "+919876543210".to_string(),
        amount: 300000,
        idempotency_key: "payment_2".to_string(),
    };

    let err = service.pay_by_phone(user_id, req2).await.unwrap_err();
    assert!(matches!(err, PaymentError::DailyLimitExceeded));
}