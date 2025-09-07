// In transaction/service.rs
pub async fn get_filtered_history(
    &self,
    user_id: Uuid,
    limit: i64,
    offset: i64,
    search: Option<&str>,
    tx_type: Option<&str>,
) -> Result<Vec<TransactionItem>, sqlx::Error> {
    let mut query = r#"
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
        WHERE (tj.from_user_id = $1 OR tj.to_user_id = $1)
    "#.to_string();

    let mut params: Vec<Box<dyn sqlx::Encode<_> + Send>> = vec![Box::new(user_id)];

    if let Some(search_term) = search {
        query.push_str(&format!(" AND (u_from.name ILIKE ${} OR u_to.name ILIKE ${})", params.len() + 1, params.len() + 2));
        params.push(Box::new(format!("%{}%", search_term)));
        params.push(Box::new(format!("%{}%", search_term)));
    }

    if let Some(t) = tx_type {
        if t == "sent" {
            query.push_str(&format!(" AND tj.from_user_id = ${}", params.len() + 1));
            params.push(Box::new(user_id));
        } else if t == "received" {
            query.push_str(&format!(" AND tj.to_user_id = ${}", params.len() + 1));
            params.push(Box::new(user_id));
        }
    }

    query.push_str(&format!(" ORDER BY tj.created_at DESC LIMIT ${} OFFSET ${}", params.len() + 1, params.len() + 2));
    params.push(Box::new(limit));
    params.push(Box::new(offset));

    let mut query_builder = sqlx::query_as::<_, TransactionItem>(&query);
    for param in params {
        query_builder = query_builder.bind(param);
    }

    query_builder.fetch_all(&self.db).await
}