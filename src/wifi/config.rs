use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WifiConfig {
    #[serde(default)]
    pub device_name: String,
    #[serde(default)]
    pub channels: Vec<u64>,
    #[serde(default)]
    pub channel_mod_freq_ms: u64,
}

impl Default for WifiConfig {
    fn default() -> Self {
        WifiConfig {
            device_name: "".to_string(),
            channels: default_channels(),
            channel_mod_freq_ms: default_channel_mod_freq_ms(),
        }
    }
}

fn default_channels() -> Vec<u64> {
    vec![1, 6, 11]
}

fn default_channel_mod_freq_ms() -> u64 {
    300
}
