// src/auth/crypto.rs

use hmac::{Hmac, Mac};
use sha2::Sha256;
use rand::Rng;
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub fn hash_mobile(mobile: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(mobile.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id as string
    pub exp: usize,
    pub device_fingerprint: Option<String>,
}

pub fn create_jwt(
    user_id: &Uuid,
    device_fingerprint: Option<&str>,
    secret: &str,
    expiry_secs: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (SystemTime::now() + std::time::Duration::from_secs(expiry_secs))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        device_fingerprint: device_fingerprint.map(|s| s.to_string()),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::HS256];
    validation.validate_exp = true;
    validation.validate_nbf = true;

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)?;
    Ok(token_data.claims)
}