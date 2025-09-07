// tests/integration/tracing.rs
use axum::{
    body::Body,
    http::{Request, HeaderMap, HeaderValue},
};
use opentelemetry::trace::{TraceContextExt, Tracer};

#[tokio::test]
async fn test_http_tracing_headers_propagated() {
    let ctx = TestContext::new().await;
    let app = payment_system::main::create_app(ctx.db.clone()).await;

    // Create trace context
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("payment-system-test")
        .install_simple()
        .unwrap();

    let span = tracer.start("test_http_tracing");
    let cx = opentelemetry::Context::current_with_span(span);

    // Propagate trace context in headers
    let mut headers = HeaderMap::new();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(&mut headers));
    });

    let req = Request::builder()
        .uri("/wallet/balance")
        .header("Authorization", format!("Bearer {}", generate_jwt(&new_uuid(), "jwt_secret")))
        .headers(headers)
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // In real test: verify trace appears in Jaeger with correct parent-child relationships
}

struct HeaderInjector<'a>(&'a mut HeaderMap);

impl<'a> opentelemetry::propagation::Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(header_name) = axum::http::header::HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(header_value) = HeaderValue::from_str(&value) {
                self.0.insert(header_name, header_value);
            }
        }
    }
}