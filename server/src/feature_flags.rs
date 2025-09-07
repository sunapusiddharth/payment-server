// src/feature_flags.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone)]
pub struct FeatureFlags {
    flags: Arc<RwLock<HashMap<String, bool>>>,
}

impl FeatureFlags {
    pub fn new() -> Self {
        let mut flags = HashMap::new();
        flags.insert("new_payment_flow".to_string(), false);
        flags.insert("enhanced_fraud_detection".to_string(), false);
        Self {
            flags: Arc::new(RwLock::new(flags)),
        }
    }

    pub async fn is_enabled(&self, feature: &str) -> bool {
        let flags = self.flags.read().await;
        *flags.get(feature).unwrap_or(&false)
    }

    pub async fn enable(&self, feature: &str) {
        let mut flags = self.flags.write().await;
        flags.insert(feature.to_string(), true);
    }

    pub async fn disable(&self, feature: &str) {
        let mut flags = self.flags.write().await;
        flags.insert(feature.to_string(), false);
    }
}