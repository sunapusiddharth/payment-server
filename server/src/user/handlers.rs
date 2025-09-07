// src/user/handlers.rs
pub async fn get_profile(
    Extension(user_service): Extension<Arc<UserService>>,
    user_id: Uuid,
) -> Result<Json<UserProfile>, (StatusCode, Json<Value>)> {
    let profile = user_service.get_profile(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(profile))
}