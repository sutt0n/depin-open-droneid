use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Default, Deserialize)]
pub struct BluetoothConfig {
    #[serde(default)]
    pub device_name: String,
}
