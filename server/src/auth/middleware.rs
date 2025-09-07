// src/auth/middleware.rs

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{StatusCode, HeaderValue},
};
use crate::auth::crypto::validate_jwt;
use tracing::error;

pub async fn jwt_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let auth_header = req.headers().get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(|s| s.to_string()));

    let token = match auth_header {
        Some(t) => t,
        None => return Err((StatusCode::UNAUTHORIZED, "Missing Authorization header")),
    };

    let secret = std::env::var("JWT_SECRET").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Server misconfigured"))?;

    let claims = validate_jwt(&token, &secret)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

    // Optional: Attach user_id to request extensions
    let mut req = req;
    req.extensions_mut().insert(claims.sub.clone());

    // Optional: Validate device fingerprint if present
    if let Some(device_fp) = claims.device_fingerprint {
        if let Some(req_fp) = req.headers().get("X-Device-Fingerprint").and_then(|v| v.to_str().ok()) {
            if req_fp != device_fp {
                return Err((StatusCode::FORBIDDEN, "Device mismatch"));
            }
        }
    }

    Ok(next.run(req).await)
}