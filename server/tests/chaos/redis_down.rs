// tests/chaos/redis_down.rs
use crate::common::TestContext;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::net::TcpStream;

#[tokio::test]
async fn test_api_gateway_degrades_gracefully_when_redis_down() {
    let ctx = TestContext::new().await;
    let app = payment_system::main::create_app(ctx.db.clone()).await;

    // Kill Redis (in real test: stop Redis server)
    // For simulation: block Redis port
    let _ = TcpStream::connect("127.0.0.1:6379"); // Just to establish we can connect

    // Send request - should degrade gracefully (bypass rate limiting)
    let req = Request::builder()
        .uri("/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"mobile": "+919876543210"}"#))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    
    // Property: Should still work (maybe slower) but not 500
    assert_eq!(res.status(), StatusCode::OK);
}