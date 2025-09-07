
// src/main.rs

use axum::{
    routing::{post, get},
    Router,
    Extension,
    http::Request,
    middleware,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use opentelemetry::sdk::trace as sdktrace;
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry::trace::Tracer;


mod auth;
mod wallet;
mod payment;
mod middleware;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize services
    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let redis_client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();

    let wallet_service = std::sync::Arc::new(wallet::WalletService::new(pool.clone()));
    let auth_service = std::sync::Arc::new(auth::AuthService::new(
        pool.clone(),
        std::env::var("JWT_SECRET").unwrap(),
        std::env::var("OTP_SECRET").unwrap(),
        std::sync::Arc::new(auth::MockSmsClient {}),
    ));
    let payment_service = std::sync::Arc::new(payment::PaymentService::new(
        pool.clone(),
        wallet_service.clone(),
        std::env::var("OTP_SECRET").unwrap(),
        std::sync::Arc::new(payment::MockNatsClient {}),
    ));

    // Build app
    let app = Router::new()
        .route("/auth/register", post(auth::handlers::register))
        .route("/auth/verify-otp", post(auth::handlers::verify_otp))
        .route("/auth/refresh", post(auth::handlers::refresh))
        .route("/auth/logout", post(auth::handlers::logout))
        .route("/wallet/balance", get(wallet::handlers::get_balance))
        .route("/pay/phone", post(payment::handlers::pay_by_phone))
        .route("/pay/qr", post(payment::handlers::pay_by_qr))
        .route("/health", get(health))
        .route("/metrics", get(metrics_handler))

        .route("/qr/:user_id.png", get(qr::handlers::get_qr_png))
.route("/qr/:user_id.svg", get(qr::handlers::get_qr_svg))
.route("/transactions", get(transaction::handlers::get_transactions))
.route("/contacts", get(contact::handlers::get_contacts))
.route("/user/profile", get(user::handlers::get_profile))



        // Layer middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::rate_limit::RateLimitLayer::new(
                    redis_client.clone(),
                    100, // 100 requests per minute per IP
                ))
                .layer(middleware::request_id::RequestIdLayer)
                .layer(Extension(auth_service))
                .layer(Extension(wallet_service))
                .layer(Extension(payment_service))
                .layer(Extension(std::sync::Arc::new(qr::service::QrService::new())))
                .layer(Extension(Arc::new(TransactionService::new(pool.clone()))))
                .layer(Extension(Arc::new(ContactService::new(pool.clone()))))
        )

        // JWT middleware for protected routes
        .route_layer(
            middleware::from_fn_with_state(
                (),
                middleware::jwt::jwt_middleware,
            ),
        )
        .with_state(redis_client);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();


        //ws server
        // Add to main.rs
let ws_server = Arc::new(WsServer::new());
let ws_server_clone = ws_server.clone();

// Start WebSocket server
tokio::spawn(async move {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3003").await.unwrap();
    println!("WebSocket server listening on ws://localhost:3003");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let ws_server = ws_server_clone.clone();
        
        tokio::spawn(async move {
            let user_id = "user123".to_string(); // In real app: authenticate via JWT in URL param
            let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            ws_server.handle_connection(user_id, ws_stream).await;
        });
    }
});

// After payment success, send notification
// In payment/service.rs, after tx.commit():
let notification = format!(
    r#"{{"type":"payment","tx_id":"{}","amount":{},"status":"Success"}}"#,
    tx_id, req.amount
);
ws_server.send_notification(&from_user_id.to_string(), &notification).await;
}

async fn health() -> &'static str {
    "OK"
}

async fn metrics_handler() -> String {
    // Expose Prometheus metrics
    use metrics_exporter_prometheus::PrometheusBuilder;
    static PROMETHEUS_RECORDER: std::sync::Once = std::sync::Once::new();
    PROMETHEUS_RECORDER.call_once(|| {
        PrometheusBuilder::new()
            .install()
            .expect("failed to install Prometheus recorder");
    });
    metrics_exporter_prometheus::PrometheusHandle::global().render()
}


fn init_tracer() -> Result<sdktrace::Tracer, Box<dyn std::error::Error>> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("localhost:6831")
        .with_process(opentelemetry_jaeger::Process {
            service_name: "payment-gateway".to_string(),
            tags: Vec::new(),
        })
        .init()?;

    let provider = sdktrace::TracerProvider::builder()
        .with_config(sdktrace::Config::default().with_resource(Resource::new(vec![
            KeyValue::new("service.name", "payment-gateway"),
        ])))
        .with_simple_exporter(exporter)
        .build();

    Ok(provider.get_tracer("gateway-tracer"))
}