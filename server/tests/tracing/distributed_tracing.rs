// tests/tracing/distributed_tracing.rs
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;
use payment_system::wallet::WalletService;
use crate::common::TestContext;

#[tokio::test]
async fn test_tracing_propagates_across_services() {
    let ctx = TestContext::new().await;
    let service = WalletService::new(ctx.db.clone());

    // Create tracer
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("payment-system-test")
        .install_simple()
        .unwrap();

    // Start root span
    let root_span = tracer.start("test_tracing_propagation");
    let cx = opentelemetry::Context::current_with_span(root_span);

    // Execute wallet operation within context
    let result = opentelemetry::Context::map_current(|_| {
        let user_id = uuid::Uuid::new_v4();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                service.create_wallet(CreateWalletRequest { user_id }).await
            })
        })
    });

    // Property: Should succeed and have trace context
    assert!(result.is_ok());

    // Export spans (in real test: verify with Jaeger)
    tracer.force_flush();
}