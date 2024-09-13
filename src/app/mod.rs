mod config;
pub mod error;

use std::{collections::HashMap, sync::Arc};

pub use config::*;

use sqlx::{Pool, Postgres};
use tokio::sync::Mutex;

use crate::{drone::Drone, mqtt_client::MqttClient};

use self::error::ApplicationError;

#[derive(Clone)]
pub struct TrebuchetApp {
    _config: AppConfig,
    //bluetooth: BluetoothConfig,
    _pool: Pool<Postgres>,
    pub drones: Arc<Mutex<HashMap<String, Drone>>>,
    mqtt_client: MqttClient,
}

impl TrebuchetApp {
    pub async fn init(pool: Pool<Postgres>, config: AppConfig) -> anyhow::Result<Self> {
        let mqtt_client = MqttClient::init(config.mqtt.clone()).await?;
        Ok(Self {
            _config: config,
            _pool: pool,
            mqtt_client,
            drones: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn run_eventloop(&self) -> anyhow::Result<(), ApplicationError> {
        self.mqtt_client.run_eventloop().await?;

        Ok(())
    }

    pub async fn send_payload(&self, payload: Vec<u8>) -> anyhow::Result<(), ApplicationError> {
        let _ = self.mqtt_client.publish(payload).await?;

        Ok(())
    }

    //pub async fn insert_drone
    //pub async fn submit_drone_for_payout

    //pub async fn handle_drone_mqtt(&self, message: DronePayload) -> anyhow::Result<()> {
    //    println!("Handling drone mqtt message: {:?}", message);
    //
    //    Ok(())
    //}
}
