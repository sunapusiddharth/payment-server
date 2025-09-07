// src/user/service.rs
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserService {
    db: PgPool,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub mobile: String,
    pub kyc_tier: String,
    pub daily_limit_used: i64,
    pub daily_limit_max: i64,
}

impl UserService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, sqlx::Error> {
        let user = sqlx::query!(
            "SELECT mobile_hash FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

        let limits = sqlx::query!(
            "SELECT kyc_tier, amount_used FROM daily_limits WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;

        let kyc_tier = limits.as_ref().map(|l| l.kyc_tier.clone().unwrap_or("basic".to_string())).unwrap_or("basic".to_string());
        let amount_used = limits.map(|l| l.amount_used.unwrap_or(0)).unwrap_or(0);
        let limit_max = if kyc_tier == "full" { 1000000 } else { 100000 }; // in paise

        Ok(UserProfile {
            mobile: mask_mobile(&user.mobile_hash),
            kyc_tier,
            daily_limit_used: amount_used,
            daily_limit_max: limit_max,
        })
    }
}