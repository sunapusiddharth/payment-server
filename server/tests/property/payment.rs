// tests/property/payment.rs
use proptest::prelude::*;
use crate::common::{TestContext, new_uuid};

proptest! {
    #[test]
    fn test_payment_idempotency(
        amount in 1..500_000u64,
        idempotency_key in "[a-z0-9]{36}"
    ) {
        let ctx = TestContext::new().await;
        let wallet_service = Arc::new(payment_system::wallet::WalletService::new(ctx.db.clone()));
        let service = PaymentService::new(
            ctx.db.clone(),
            wallet_service.clone(),
            "otp_secret".to_string(),
            Arc::new(MockNatsClient::new()),
        );

        // Create users
        let sender_id = new_uuid();
        let receiver_id = new_uuid();
        let receiver_mobile = "+919876543210";

        // Setup
        wallet_service.create_wallet(CreateWalletRequest { user_id: sender_id }).await.unwrap();
        wallet_service.credit(&CreditDebitRequest {
            user_id: sender_id,
            amount: 1_000_000, // ₹10,000
            idempotency_key: "setup".to_string(),
        }).await.unwrap();

        let mobile_hash = payment_system::auth::crypto::hash_mobile(receiver_mobile, "otp_secret");
        sqlx::query!(
            "INSERT INTO users (id, mobile_hash) VALUES ($1, $2)",
            receiver_id,
            mobile_hash
        )
        .execute(&ctx.db)
        .await
        .unwrap();

        // First payment
        let req = PayByPhoneRequest {
            to_mobile: receiver_mobile.to_string(),
            amount,
            idempotency_key: idempotency_key.clone(),
        };

        let first_result = service.pay_by_phone(sender_id, req.clone()).await;

        // Second payment with same idempotency key
        let second_result = service.pay_by_phone(sender_id, req).await;

        // Property: First succeeds, second fails with DuplicateIdempotencyKey
        prop_assert!(first_result.is_ok());
        prop_assert!(matches!(second_result, Err(PaymentError::DuplicateIdempotencyKey)));
    }
}