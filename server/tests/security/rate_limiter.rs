// tests/security/rate_limit.rs
use crate::common::TestContext;
use payment_system::main;
use std::time::Duration;

#[tokio::test]
async fn test_rate_limiting_blocks_requests() {
    let ctx = TestContext::new().await;
    let app = main::create_app(ctx.db.clone()).await;

    let client_ip = "127.0.0.1";

    // Send 101 requests (limit is 100/min)
    for i in 0..101 {
        let req = Request::builder()
            .uri("/auth/register")
            .header("X-Forwarded-For", client_ip)
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"mobile": "+919876543210"}"#))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();

        if i == 100 {
            // 101st request should be blocked
            assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
        } else {
            assert_eq!(res.status(), StatusCode::OK);
        }
    }
}