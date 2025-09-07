// src/transaction/service.rs

pub struct TransactionService {
    db: PgPool,
}

impl TransactionService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_history(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TransactionItem>, sqlx::Error> {
        sqlx::query_as!(
            TransactionItem,
            r#"
            SELECT
                tj.tx_id,
                tj.amount,
                tj.created_at as "timestamp!",
                tj.status,
                CASE
                    WHEN tj.from_user_id = $1 THEN u_to.mobile_hash
                    ELSE u_from.mobile_hash
                END as counterparty_mobile,
                CASE
                    WHEN tj.from_user_id = $1 THEN 'sent'
                    ELSE 'received'
                END as "transaction_type!"
            FROM transaction_journal tj
            LEFT JOIN users u_from ON tj.from_user_id = u_from.id
            LEFT JOIN users u_to ON tj.to_user_id = u_to.id
            WHERE tj.from_user_id = $1 OR tj.to_user_id = $1
            ORDER BY tj.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
    }
}