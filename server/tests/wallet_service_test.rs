// tests/wallet_service_test.rs

use your_crate::wallet_service::{WalletService, CreateWalletRequest, CreditDebitRequest};
use sqlx::{PgPool, PgConnection, Connection, Executor};
use uuid::Uuid;
use std::str::FromStr;

#[tokio::test]
async fn test_create_and_credit_wallet() {
    let pool = setup_test_db().await;
    let service = WalletService::new(pool);

    let user_id = Uuid::new_v4();
    
    // Create
    let req = CreateWalletRequest { user_id };
    let wallet = service.create_wallet(req).await.unwrap();
    assert_eq!(wallet.balance, 0);

    // Credit
    let credit_req = CreditDebitRequest {
        user_id,
        amount: 10000, // ₹100
        idempotency_key: Uuid::new_v4().to_string(),
    };
    let updated = service.credit(&credit_req).await.unwrap();
    assert_eq!(updated.balance, 10000);

    // Check balance
    let balance = service.get_balance(&user_id).await.unwrap();
    assert_eq!(balance, 10000);
}

#[tokio::test]
async fn test_debit_insufficient_funds() {
    let pool = setup_test_db().await;
    let service = WalletService::new(pool);

    let user_id = Uuid::new_v4();
    service.create_wallet(CreateWalletRequest { user_id }).await.unwrap();

    let req = CreditDebitRequest {
        user_id,
        amount: 50000, // ₹500
        idempotency_key: Uuid::new_v4().to_string(),
    };

    let err = service.debit(&req).await.unwrap_err();
    assert!(matches!(err, WalletError::InsufficientBalance));
}

async fn setup_test_db() -> PgPool {
    let mut conn = PgConnection::connect(&std::env::var("DATABASE_URL_TEST").unwrap())
        .await
        .unwrap();
    conn.execute(include_str!("../../migrations/001_wallets.up.sql")).await.unwrap();
    PgPool::connect(&std::env::var("DATABASE_URL_TEST").unwrap()).await.unwrap()
}