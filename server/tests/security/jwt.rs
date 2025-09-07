// tests/security/jwt.rs
use crate::common::{TestContext, new_uuid};
use payment_system::main;

#[tokio::test]
async fn test_invalid_jwt_rejected() {
    let ctx = TestContext::new().await;
    let app = main::create_app(ctx.db.clone()).await;

    let req = Request::builder()
        .uri("/wallet/balance")
        .header("Authorization", "Bearer invalid.jwt.token")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_expired_jwt_rejected() {
    let ctx = TestContext::new().await;
    let app = main::create_app(ctx.db.clone()).await;

    // Create expired JWT (1 second expiry)
    let user_id = new_uuid();
    let jwt = payment_system::auth::crypto::create_jwt(&user_id, None, "jwt_secret", 1).unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; // Wait for expiry

    let req = Request::builder()
        .uri("/wallet/balance")
        .header("Authorization", format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}