// src/transaction/handlers.rs
pub async fn get_transactions(
    Extension(tx_service): Extension<Arc<TransactionService>>,
    user_id: Uuid,
    query: Query<HashMap<String, String>>,
) -> Result<Json<Vec<TransactionItem>>, (StatusCode, Json<Value>)> {
    let limit = query.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = query.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    let search = query.get("search").map(|s| s.to_lowercase());
    let tx_type = query.get("type").map(|s| s.as_str()); // "sent", "received", or "all"

    let transactions = tx_service.get_filtered_history(user_id, limit, offset, search.as_deref(), tx_type)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    // Mask mobile numbers
    let masked: Vec<TransactionItem> = transactions.into_iter().map(|mut t| {
        if let Some(mobile_hash) = t.counterparty_mobile {
            t.counterparty_mobile = Some(mask_mobile(&mobile_hash));
        }
        t
    }).collect();

    Ok(Json(masked))
}