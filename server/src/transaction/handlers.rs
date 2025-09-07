// src/transaction/handlers.rs

pub async fn get_transactions(
    Extension(tx_service): Extension<Arc<TransactionService>>,
    user_id: Uuid,
    query: Query<HashMap<String, String>>,
) -> Result<Json<Vec<TransactionItem>>, (StatusCode, Json<Value>)> {
    let limit = query.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = query.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    let transactions = tx_service.get_history(user_id, limit, offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    // Mask mobile numbers
    let masked: Vec<TransactionItem> = transactions.into_iter().map(|mut t| {
        if let Some(mobile_hash) = t.counterparty_mobile {
            // In real system, you’d store last 2 digits or use a reversible mask
            t.counterparty_mobile = Some("+91******3210".to_string());
        }
        t
    }).collect();

    Ok(Json(masked))
}