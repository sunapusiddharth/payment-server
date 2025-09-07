// src/middleware/rate_limit.rs

use axum::{
    middleware::Next,
    response::Response,
    http::{Request, StatusCode},
};
use redis::{AsyncCommands, Client};
use std::sync::Arc;
use tracing::warn;

pub struct RateLimitLayer {
    redis: Client,
    max_requests: i32,
}

impl RateLimitLayer {
    pub fn new(redis: Client, max_requests: i32) -> Self {
        Self { redis, max_requests }
    }
}

impl<S> tower::Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            redis: self.redis.clone(),
            max_requests: self.max_requests,
        }
    }
}

pub struct RateLimitService<S> {
    inner: S,
    redis: Client,
    max_requests: i32,
}

impl<S, B> tower::Service<Request<B>> for RateLimitService<S>
where
    S: tower::Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let redis = self.redis.clone();
        let max_requests = self.max_requests;

        Box::pin(async move {
            let ip = req.headers().get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("127.0.0.1");

            let key = format!("rate_limit:{}", ip);
            let mut conn = redis.get_async_connection().await.map_err(|e| {
                warn!(error = %e, "Failed to connect to Redis for rate limiting");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let count: i32 = conn.get(&key).await.unwrap_or(0);
            if count >= max_requests {
                return Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into());
            }

            let _: () = conn.set_ex(&key, count + 1, 60).await.unwrap_or(());

            inner.call(req).await
        })
    }
}