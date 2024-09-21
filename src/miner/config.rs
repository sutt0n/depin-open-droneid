use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerConfig {
    #[serde(default)]
    pub wallet_address: String,
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self {
            wallet_address: "".to_string(),
        }
    }
}
