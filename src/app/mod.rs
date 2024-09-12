mod config;
pub mod error;

use std::{collections::HashMap, sync::Arc};

pub use config::*;

use sqlx::{Pool, Postgres};
use tokio::sync::Mutex;

use crate::drone::Drone;

#[derive(Clone)]
pub struct TrebuchetApp {
    _config: AppConfig,
    //bluetooth: BluetoothConfig,
    _pool: Pool<Postgres>,
    pub drones: Arc<Mutex<HashMap<String, Drone>>>,
}

impl TrebuchetApp {
    pub async fn init(pool: Pool<Postgres>, config: AppConfig) -> anyhow::Result<Self> {
        Ok(Self {
            _config: config,
            _pool: pool,
            drones: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    //pub async fn insert_drone
    //pub async fn submit_drone_for_payout

    //pub async fn handle_drone_mqtt(&self, message: DronePayload) -> anyhow::Result<()> {
    //    println!("Handling drone mqtt message: {:?}", message);
    //
    //    Ok(())
    //}
}
