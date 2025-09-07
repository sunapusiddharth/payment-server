// src/middleware/request_id.rs

use axum::{
    middleware::Next,
    response::Response,
    http::{Request, HeaderValue},
};
use uuid::Uuid;
use tracing::Span;

pub struct RequestIdLayer;

impl<S> tower::Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

pub struct RequestIdService<S> {
    inner: S,
}

impl<S, B> tower::Service<Request<B>> for RequestIdService<S>
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

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let request_id = Uuid::new_v4().to_string();
        Span::current().record("request_id", &request_id);

        let header_value = HeaderValue::from_str(&request_id).unwrap();
        req.headers_mut().insert("X-Request-ID", header_value);

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            inner.call(req).await
        })
    }
}