// tests/deployment/health_check.rs
use axum::http::StatusCode;

#[tokio::test]
async fn test_health_check_during_deployment() {
    let ctx = TestContext::new().await;
    let app = payment_system::main::create_app(ctx.db.clone()).await;

    // Simulate deployment (stop accepting new connections)
    let deployment_in_progress = Arc::new(AtomicBool::new(false));

    // Health check should return 503 during deployment
    {
        let deployment_in_progress = deployment_in_progress.clone();
        deployment_in_progress.store(true, Ordering::SeqCst);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    // Health check should return 200 when deployment complete
    {
        deployment_in_progress.store(false, Ordering::SeqCst);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}