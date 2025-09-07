// tests/contract/api_contract.rs
use serde_json::Value;
use std::fs;

#[tokio::test]
async fn test_api_contracts_honored() {
    let openapi_spec: Value = serde_json::from_str(
        &fs::read_to_string("tests/contract/openapi.json").unwrap()
    ).unwrap();

    // Test /auth/register contract
    let client = reqwest::Client::new();
    
    // Test valid request
    let valid_req = serde_json::json!({
        "mobile": "+919876543210"
    });
    
    let res = client.post("http://localhost:3000/auth/register")
        .json(&valid_req)
        .send()
        .await
        .unwrap();
    
    assert_eq!(res.status(), 200);
    
    let body: Value = res.json().await.unwrap();
    assert!(body["message"].as_str().unwrap().contains("OTP sent"));

    // Test invalid request (wrong mobile format)
    let invalid_req = serde_json::json!({
        "mobile": "12345"
    });
    
    let res = client.post("http://localhost:3000/auth/register")
        .json(&invalid_req)
        .send()
        .await
        .unwrap();
    
    // Property: Should return 400 with validation error
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("Invalid Indian mobile number"));
}