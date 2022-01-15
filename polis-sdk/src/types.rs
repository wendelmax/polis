use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    pub endpoint: String,
    pub timeout: u64,
}
