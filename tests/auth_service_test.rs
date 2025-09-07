// tests/auth_service_test.rs

#[tokio::test]
async fn test_register_and_login() {
    let pool = setup_test_db().await;
    let sms_client = Arc::new(MockSmsClient {});
    let service = AuthService::new(pool, "jwt_secret_32_chars_min".to_string(), "otp_secret".to_string(), sms_client);

    let mobile = "+919876543210".to_string();

    // Register
    service.register(RegisterRequest { mobile: mobile.clone() }).await.unwrap();

    // Verify OTP — we need to fetch the OTP from DB for test
    let otp = get_otp_for_mobile(&mobile, &service).await.unwrap();

    let resp = service.verify_otp(VerifyOtpRequest {
        mobile: mobile.clone(),
        otp,
        device_fingerprint: Some("device_123".to_string()),
    }).await.unwrap();

    assert!(!resp.access_token.is_empty());
    assert!(!resp.refresh_token.is_empty());
    assert_eq!(resp.expires_in, 900);
}

#[tokio::test]
async fn test_refresh_token() {
    // ... similar setup
    let resp = service.refresh_token(&refresh_token).await.unwrap();
    assert_ne!(resp.refresh_token, refresh_token); // new token issued
}

struct MockSmsClient;
#[async_trait::async_trait]
impl SmsClient for MockSmsClient {
    async fn send_otp(&self, _mobile: &str, _otp: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}