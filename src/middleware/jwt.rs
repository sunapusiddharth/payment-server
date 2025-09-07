// src/middleware/jwt.rs

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{StatusCode, HeaderMap},
    Extension,
};
use crate::auth::crypto::validate_jwt;
use tracing::error;

pub async fn jwt_middleware(
    Extension(secret): Extension<String>, // inject JWT_SECRET
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let auth_header = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(|s| s.to_string()));

    let token = match auth_header {
        Some(t) => t,
        None => return Err((StatusCode::UNAUTHORIZED, "Missing Authorization header")),
    };

    let claims = validate_jwt(&token, &secret)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

    // Extract user_id
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user_id in token"))?;

    // Insert into request extensions
    req.extensions_mut().insert(user_id);

    // Optional: Validate device fingerprint
    if let Some(device_fp) = claims.device_fingerprint {
        if let Some(req_fp) = headers.get("X-Device-Fingerprint").and_then(|v| v.to_str().ok()) {
            if req_fp != device_fp {
                return Err((StatusCode::FORBIDDEN, "Device mismatch"));
            }
        }
    }

    Ok(next.run(req).await)
}