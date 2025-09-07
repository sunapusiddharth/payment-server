// tests/integration/api_gateway.rs
use crate::common::{TestContext, new_uuid, generate_jwt};
use payment_system::main;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_protected_route_requires_jwt() {
    let ctx = TestContext::new().await;
    let app = main::create_app(ctx.db.clone()).await; // You'll need to expose this from main.rs

    let req = Request::builder()
        .uri("/wallet/balance")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_balance_with_valid_jwt() {
    let ctx = TestContext::new().await;
    let app = main::create_app(ctx.db.clone()).await;

    // Create user and wallet
    let user_id = new_uuid();
    let wallet_service = payment_system::wallet::WalletService::new(ctx.db.clone());
    wallet_service.create_wallet(CreateWalletRequest { user_id }).await.unwrap();
    wallet_service.credit(&CreditDebitRequest {
        user_id,
        amount: 25000,
        idempotency_key: "test".to_string(),
    }).await.unwrap();

    // Generate JWT
    let jwt = generate_jwt(&user_id, "jwt_secret");

    let req = Request::builder()
        .uri("/wallet/balance")
        .header("Authorization", format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let balance: i64 = serde_json::from_slice(&body).unwrap();
    assert_eq!(balance, 25000);
}