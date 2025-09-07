// benches/payment_benchmark.rs
use criterion::{Criterion, criterion_group, criterion_main};
use tokio::runtime::Runtime;
use payment_system::payment::{PaymentService, models::PayByPhoneRequest};
use payment_system::wallet::{WalletService, models::CreateWalletRequest};
use sqlx::PgPool;
use std::sync::Arc;

fn payment_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(PgPool::connect("postgres://localhost/payment_system_test")).unwrap();
    
    let wallet_service = Arc::new(WalletService::new(pool.clone()));
    let service = Arc::new(PaymentService::new(
        pool.clone(),
        wallet_service.clone(),
        "otp_secret".to_string(),
        Arc::new(MockNatsClient::new()),
    ));

    // Setup test data
    rt.block_on(async {
        let sender_id = uuid::Uuid::new_v4();
        let receiver_id = uuid::Uuid::new_v4();
        let receiver_mobile = "+919876543210";

        wallet_service.create_wallet(CreateWalletRequest { user_id: sender_id }).await.unwrap();
        wallet_service.credit(&payment_system::wallet::models::CreditDebitRequest {
            user_id: sender_id,
            amount: 1_000_000,
            idempotency_key: "setup".to_string(),
        }).await.unwrap();

        let mobile_hash = payment_system::auth::crypto::hash_mobile(receiver_mobile, "otp_secret");
        sqlx::query!(
            "INSERT INTO users (id, mobile_hash) VALUES ($1, $2)",
            receiver_id,
            mobile_hash
        )
        .execute(&pool)
        .await
        .unwrap();
    });

    c.bench_function("pay_by_phone", |b| b.to_async(&rt).iter(|| {
        let service = service.clone();
        let sender_id = uuid::Uuid::new_v4(); // Use different sender each time
        let req = PayByPhoneRequest {
            to_mobile: "+919876543210".to_string(),
            amount: 10000,
            idempotency_key: uuid::Uuid::new_v4().to_string(),
        };
        async move {
            let _ = service.pay_by_phone(sender_id, req).await;
        }
    }));
}

criterion_group!(benches, payment_benchmark);
criterion_main!(benches);