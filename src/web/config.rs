use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub host: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: "".to_string(),
        }
    }
}

fn default_port() -> u16 {
    8080
}
