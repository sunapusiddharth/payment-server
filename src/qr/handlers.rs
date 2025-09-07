// src/qr/handlers.rs

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;
use crate::qr::service::QrService;

pub async fn get_qr_png(
    Path(user_id): Path<Uuid>,
    Extension(qr_service): Extension<std::sync::Arc<QrService>>,
) -> Result<Response, (StatusCode, String)> {
    let png_data = qr_service.generate_qr_png(&user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("QR gen failed: {}", e)))?;

    Ok((
        [
            ("Content-Type", "image/png"),
            ("Cache-Control", "public, max-age=31536000"), // 1 year
            ("X-Content-Type-Options", "nosniff"),
        ],
        png_data,
    ).into_response())
}

pub async fn get_qr_svg(
    Path(user_id): Path<Uuid>,
    Extension(qr_service): Extension<std::sync::Arc<QrService>>,
) -> Result<Response, (StatusCode, String)> {
    let svg_data = qr_service.generate_qr_svg(&user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("QR gen failed: {}", e)))?;

    Ok((
        [
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=31536000"),
            ("X-Content-Type-Options", "nosniff"),
        ],
        svg_data,
    ).into_response())
}