use serde::{Deserialize, Serialize};

use crate::{
    bluetooth::BluetoothConfig, mqtt_client::MqttClientConfig, web::WebConfig, wifi::WifiConfig,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub bluetooth: BluetoothConfig,
    #[serde(default)]
    pub wifi: WifiConfig,
    #[serde(default)]
    pub web: WebConfig,
    #[serde(default)]
    pub mqtt: MqttClientConfig,
}
