// src/contact/handlers.rs
pub async fn get_contacts(
    Extension(contact_service): Extension<Arc<ContactService>>,
    user_id: Uuid,
) -> Result<Json<Vec<Contact>>, (StatusCode, Json<Value>)> {
    let contacts = contact_service.get_contacts(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    // Mask mobile numbers
    let masked: Vec<Contact> = contacts.into_iter().map(|mut c| {
        c.mobile = mask_mobile(&c.mobile); // implement mask_mobile
        c
    }).collect();

    Ok(Json(masked))
}

fn mask_mobile(mobile_hash: &str) -> String {
    // In real app, store last 4 digits during registration
    // For demo: return fixed mask
    "+91******3210".to_string()
}