// tests/unit/wallet.rs
use crate::common::{TestContext, new_uuid};
use payment_system::wallet::{WalletService, models::*};

#[tokio::test]
async fn test_create_wallet() {
    let ctx = TestContext::new().await;
    let service = WalletService::new(ctx.db.clone());

    let user_id = new_uuid();
    let req = CreateWalletRequest { user_id };

    let wallet = service.create_wallet(req).await.unwrap();

    assert_eq!(wallet.user_id, user_id);
    assert_eq!(wallet.balance, 0);
}

#[tokio::test]
async fn test_credit_wallet() {
    let ctx = TestContext::new().await;
    let service = WalletService::new(ctx.db.clone());

    let user_id = new_uuid();
    service.create_wallet(CreateWalletRequest { user_id }).await.unwrap();

    let req = CreditDebitRequest {
        user_id,
        amount: 10000,
        idempotency_key: "test_key_1".to_string(),
    };

    let wallet = service.credit(&req).await.unwrap();

    assert_eq!(wallet.balance, 10000);
}

#[tokio::test]
async fn test_debit_insufficient_balance() {
    let ctx = TestContext::new().await;
    let service = WalletService::new(ctx.db.clone());

    let user_id = new_uuid();
    service.create_wallet(CreateWalletRequest { user_id }).await.unwrap();

    let req = CreditDebitRequest {
        user_id,
        amount: 5000,
        idempotency_key: "test_key_2".to_string(),
    };

    let err = service.debit(&req).await.unwrap_err();
    assert!(matches!(err, WalletError::InsufficientBalance));
}