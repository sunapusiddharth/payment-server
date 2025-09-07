// tests/unit/payment.rs
use crate::common::{TestContext, new_uuid};
use payment_system::payment::{PaymentService, models::*};
use mockall::mock;
use async_trait::async_trait;

mock! {
    pub NatsClient {}
    #[async_trait]
    impl NatsClient for NatsClient {
        async fn publish_fraud_event(&self, event: &payment_system::payment::models::FraudEvent) -> Result<(), Box<dyn std::error::Error>>;
    }
}

#[tokio::test]
async fn test_pay_by_phone_success() {
    let ctx = TestContext::new().await;
    let wallet_service = Arc::new(payment_system::wallet::WalletService::new(ctx.db.clone()));
    let nats_client = Arc::new(MockNatsClient::new());

    let service = PaymentService::new(
        ctx.db.clone(),
        wallet_service.clone(),
        "otp_secret".to_string(),
        nats_client,
    );

    // Create sender and receiver
    let sender_id = new_uuid();
    let receiver_mobile = "+919876543210";

    // Create wallets
    wallet_service.create_wallet(CreateWalletRequest { user_id: sender_id }).await.unwrap();
    wallet_service.credit(&CreditDebitRequest {
        user_id: sender_id,
        amount: 50000,
        idempotency_key: "credit_sender".to_string(),
    }).await.unwrap();

    // Create receiver user
    let receiver_id = new_uuid();
    let mobile_hash = payment_system::auth::crypto::hash_mobile(receiver_mobile, "otp_secret");
    sqlx::query!(
        "INSERT INTO users (id, mobile_hash) VALUES ($1, $2)",
        receiver_id,
        mobile_hash
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // Make payment
    let req = PayByPhoneRequest {
        to_mobile: receiver_mobile.to_string(),
        amount: 10000,
        idempotency_key: "payment_1".to_string(),
    };

    let resp = service.pay_by_phone(sender_id, req).await.unwrap();

    assert_eq!(resp.status, PaymentStatus::Success);
    assert_eq!(resp.amount, 10000);

    // Verify balances
    let sender_balance = wallet_service.get_balance(&sender_id).await.unwrap();
    let receiver_balance = wallet_service.get_balance(&receiver_id).await.unwrap();

    assert_eq!(sender_balance, 40000);
    assert_eq!(receiver_balance, 10000);
}