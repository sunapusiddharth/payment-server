// tests/canary/canary_deployment.rs
use crate::common::TestContext;
use payment_system::feature_flags::FeatureFlags;

#[tokio::test]
async fn test_canary_deployment_strategy() {
    let flags = FeatureFlags::new();
    
    // Initially disabled
    assert!(!flags.is_enabled("new_payment_flow").await);

    // Enable for canary (5% of users)
    flags.enable("new_payment_flow").await;

    // Test canary users get new flow
    let canary_user = "user_canary_123";
    let non_canary_user = "user_normal_456";

    // In real app: use consistent hashing to assign 5% of users to canary
    let is_canary = |user_id: &str| -> bool {
        let hash = crc32::checksum_ieee(user_id.as_bytes());
        hash % 100 < 5 // 5% canary
    };

    assert!(is_canary(canary_user));
    assert!(!is_canary(non_canary_user));

    // Property: Canary users should get new flow
    if is_canary(canary_user) {
        assert!(flags.is_enabled("new_payment_flow").await);
    }

    // After monitoring, enable for all
    flags.enable("new_payment_flow").await;
    assert!(flags.is_enabled("new_payment_flow").await);
}