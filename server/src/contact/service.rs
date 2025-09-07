// src/contact/service.rs
use sqlx::PgPool;
use uuid::Uuid;

pub struct ContactService {
    db: PgPool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Contact {
    pub user_id: Uuid,
    pub name: String,
    pub mobile: String, // masked
}

impl ContactService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_contacts(&self, user_id: Uuid) -> Result<Vec<Contact>, sqlx::Error> {
        sqlx::query_as!(
            Contact,
            r#"
            SELECT u.id as user_id, u.name, u.mobile_hash as mobile
            FROM users u
            WHERE u.id != $1
            LIMIT 50
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await
    }
}