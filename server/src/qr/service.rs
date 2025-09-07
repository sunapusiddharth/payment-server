// src/qr/service.rs

use qrcode::QrCode;
use image::{ImageBuffer, Rgba};
use std::io::Cursor;

pub struct QrService;

impl QrService {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_qr_png(&self, user_id: &uuid::Uuid) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Format: payment://user/<uuid>
        let content = format!("payment://user/{}", user_id);

        // Generate QR code
        let code = QrCode::new(content.as_bytes())?;

        // Render to 300x300 PNG
        let image = code.render::<Rgba<u8>>()
            .min_dimensions(300, 300)
            .dark_color(Rgba([0, 0, 0, 255]))   // black
            .light_color(Rgba([255, 255, 255, 255])) // white
            .build();

        // Encode to PNG
        let mut buf = Vec::new();
        let cursor = Cursor::new(&mut buf);
        image::codecs::png::PngEncoder::new(cursor).write_image(
            &image,
            image.width(),
            image.height(),
            image::ColorType::Rgba8,
        )?;

        Ok(buf)
    }

    pub fn generate_qr_svg(&self, user_id: &uuid::Uuid) -> Result<String, Box<dyn std::error::Error>> {
        let content = format!("payment://user/{}", user_id);
        let code = QrCode::new(content.as_bytes())?;
        let svg = code.render().min_dimensions(300, 300).to_string();
        Ok(svg)
    }
}