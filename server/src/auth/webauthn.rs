// src/auth/webauthn.rs
use webauthn_rs::{
    prelude::*,
    Webauthn,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct WebAuthnService {
    webauthn: Webauthn,
    db: PgPool,
}

impl WebAuthnService {
    pub fn new(domain: &str, db: PgPool) -> Self {
        let rp_id = domain;
        let rp_origin = Url::parse(&format!("https://{}", domain)).unwrap();
        let builder = WebauthnBuilder::new(rp_id, &rp_origin).unwrap();
        let webauthn = builder.build().unwrap();

        Self { webauthn, db }
    }

    pub async fn start_registration(&self, user_id: Uuid) -> Result<(RegisterPublicKeyCredential, Vec<u8>), String> {
        let user_unique_id = UserId::new(user_id.as_bytes().to_vec());
        let user_name = "user".to_string(); // In real app: use actual username
        let user_display_name = "User".to_string();

        let user = WebauthnUser::new(
            user_unique_id,
            user_name,
            user_display_name,
            0,
        );

        let (ccr, skr) = self.webauthn.start_passkey_registration(user, None, None, None).map_err(|e| e.to_string())?;
        let skr_bytes = serde_json::to_vec(&skr).map_err(|e| e.to_string())?;

        // Store skr_bytes in session or cache for later verification
        // In real app: store in Redis with TTL
        sqlx::query!(
            "INSERT INTO webauthn_sessions (user_id, session_data) VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET session_data = $2",
            user_id,
            skr_bytes
        )
        .execute(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok((ccr, skr_bytes))
    }

    pub async fn finish_registration(&self, user_id: Uuid, reg_ &str) -> Result<(), String> {
        let skr_bytes: Vec<u8> = sqlx::query_scalar!(
            "SELECT session_data FROM webauthn_sessions WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No session found".to_string())?;

        let skr: PasskeyRegistration = serde_json::from_slice(&skr_bytes).map_err(|e| e.to_string())?;
        let reg: RegisterPublicKeyCredential = serde_json::from_str(reg_data).map_err(|e| e.to_string())?;

        let passkey = self.webauthn.finish_passkey_registration(&reg, &skr).map_err(|e| e.to_string())?;

        // Store passkey in DB
        let credential_id = passkey.cred_id().clone();
        let credential = serde_json::to_vec(&passkey).map_err(|e| e.to_string())?;

        sqlx::query!(
            "INSERT INTO webauthn_credentials (user_id, credential_id, credential) VALUES ($1, $2, $3) ON CONFLICT (credential_id) DO UPDATE SET credential = $3",
            user_id,
            credential_id,
            credential
        )
        .execute(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn start_authentication(&self, user_id: Uuid) -> Result<(RequestChallengeResponse, Vec<u8>), String> {
        let creds: Vec<Vec<u8>> = sqlx::query_scalar!(
            "SELECT credential FROM webauthn_credentials WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        if creds.is_empty() {
            return Err("No credentials found".to_string());
        }

        let mut passkeys = Vec::new();
        for cred_bytes in creds {
            let passkey: Passkey = serde_json::from_slice(&cred_bytes).map_err(|e| e.to_string())?;
            passkeys.push(passkey);
        }

        let (rcr, ast) = self.webauthn.start_passkey_authentication(&passkeys).map_err(|e| e.to_string())?;
        let ast_bytes = serde_json::to_vec(&ast).map_err(|e| e.to_string())?;

        sqlx::query!(
            "INSERT INTO webauthn_sessions (user_id, session_data) VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET session_data = $2",
            user_id,
            ast_bytes
        )
        .execute(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok((rcr, ast_bytes))
    }

    pub async fn finish_authentication(&self, user_id: Uuid, auth_ &str) -> Result<(), String> {
        let ast_bytes: Vec<u8> = sqlx::query_scalar!(
            "SELECT session_data FROM webauthn_sessions WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No session found".to_string())?;

        let ast: PasskeyAuthentication = serde_json::from_slice(&ast_bytes).map_err(|e| e.to_string())?;
        let auth: PublicKeyCredential = serde_json::from_str(auth_data).map_err(|e| e.to_string())?;

        let _ = self.webauthn.finish_passkey_authentication(&auth, &ast).map_err(|e| e.to_string())?;

        Ok(())
    }
}