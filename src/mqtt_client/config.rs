use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MqttClientConfig {
    #[serde(default)]
    pub uri: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub keep_alive: u64,
    #[serde(default)]
    pub ca_cert: String,
    #[serde(default)]
    pub client_cert: String,
    #[serde(default)]
    pub client_key: String,
}

impl Default for MqttClientConfig {
    fn default() -> Self {
        Self {
            uri: "".to_string(),
            port: default_port(),
            topic: "".to_string(),
            keep_alive: default_keep_alive(),
            ca_cert: "".to_string(),
            client_cert: "".to_string(),
            client_key: "".to_string(),
        }
    }
}

fn default_keep_alive() -> u64 {
    5
}

fn default_port() -> u16 {
    1883
}
