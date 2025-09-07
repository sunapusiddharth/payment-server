// tests/property/wallet.rs
use proptest::prelude::*;
use crate::common::{TestContext, new_uuid};
use payment_system::wallet::{WalletService, models::CreditDebitRequest};

proptest! {
    #[test]
    fn test_wallet_credits_and_debits_are_commutative(
        initial_balance in 0..1_000_000u64,
        credit_amount in 1..500_000u64,
        debit_amount in 1..500_000u64
    ) {
        // Test: credit then debit should equal debit then credit
        let ctx = TestContext::new().await;
        let service = WalletService::new(ctx.db.clone());
        let user_id = new_uuid();

        // Create wallet with initial balance
        service.create_wallet(CreateWalletRequest { user_id }).await.unwrap();
        if initial_balance > 0 {
            service.credit(&CreditDebitRequest {
                user_id,
                amount: initial_balance,
                idempotency_key: "initial".to_string(),
            }).await.unwrap();
        }

        // Scenario 1: Credit then Debit
        let balance1 = {
            service.credit(&CreditDebitRequest {
                user_id,
                amount: credit_amount,
                idempotency_key: "credit1".to_string(),
            }).await.unwrap();
            let wallet = service.debit(&CreditDebitRequest {
                user_id,
                amount: debit_amount,
                idempotency_key: "debit1".to_string(),
            }).await.unwrap();
            wallet.balance
        };

        // Reset
        ctx.cleanup().await;
        service.create_wallet(CreateWalletRequest { user_id }).await.unwrap();
        if initial_balance > 0 {
            service.credit(&CreditDebitRequest {
                user_id,
                amount: initial_balance,
                idempotency_key: "initial2".to_string(),
            }).await.unwrap();
        }

        // Scenario 2: Debit then Credit
        let balance2 = {
            let wallet = service.debit(&CreditDebitRequest {
                user_id,
                amount: debit_amount,
                idempotency_key: "debit2".to_string(),
            }).await.unwrap();
            service.credit(&CreditDebitRequest {
                user_id,
                amount: credit_amount,
                idempotency_key: "credit2".to_string(),
            }).await.unwrap();
            wallet.balance
        };

        // Property: Both scenarios should result in same balance
        prop_assert_eq!(balance1, balance2);
    }

    #[test]
    fn test_wallet_never_goes_negative(amount in 1..1_000_000u64) {
        let ctx = TestContext::new().await;
        let service = WalletService::new(ctx.db.clone());
        let user_id = new_uuid();

        service.create_wallet(CreateWalletRequest { user_id }).await.unwrap();

        // Try to debit more than balance
        let result = service.debit(&CreditDebitRequest {
            user_id,
            amount,
            idempotency_key: "test".to_string(),
        }).await;

        // Property: Should fail with InsufficientBalance
        prop_assert!(matches!(result, Err(WalletError::InsufficientBalance)));
    }
}