// tests/failure/insufficient_balance.rs
use crate::common::{TestContext, new_uuid};
use payment_system::payment::{PaymentService, models::*};

#[tokio::test]
async fn test_payment_fails_with_insufficient_balance() {
    let ctx = TestContext::new().await;
    let wallet_service = Arc::new(payment_system::wallet::WalletService::new(ctx.db.clone()));
    let service = PaymentService::new(
        ctx.db.clone(),
        wallet_service.clone(),
        "otp_secret".to_string(),
        Arc::new(MockNatsClient::new()),
    );

    // Create sender with only ₹100
    let sender_id = new_uuid();
    wallet_service.create_wallet(CreateWalletRequest { user_id: sender_id }).await.unwrap();
    wallet_service.credit(&CreditDebitRequest {
        user_id: sender_id,
        amount: 10000,
        idempotency_key: "credit_1".to_string(),
    }).await.unwrap();

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

    // Try to send ₹500
    let req = PayByPhoneRequest {
        to_mobile: "+919876543210".to_string(),
        amount: 50000,
        idempotency_key: "payment_1".to_string(),
    };

    let err = service.pay_by_phone(sender_id, req).await.unwrap_err();
    assert!(matches!(err, PaymentError::WalletError(WalletError::InsufficientBalance)));
}