pub mod config;

use crate::{app::TrebuchetApp, web::DroneDto};
use mac_address::get_mac_address;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::{uuid, Uuid};

use std::f64::consts::PI;

use self::config::MinerConfig;

const EARTH_RADIUS_KM: f64 = 6371.0;

const NAMESPACE_UUID: Uuid = uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub latitude: f64,
    pub longtitude: f64,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttPayload {
    pub machine: Machine,
    pub drone: DroneDto,
}

pub async fn start_miner_task(app: TrebuchetApp, config: MinerConfig) -> anyhow::Result<()> {
    tokio::spawn(async move {
        loop {
            if !should_send_payload() {
                continue;
            }

            let drone = DroneDto::dummy();
            let (lat, lon) = generate_random_point(drone.latitude, drone.longitude, 5.0);
            let machine = Machine {
                id: generate_uuid_v5_from_mac(&get_mac_as_string().unwrap()).to_string(),
                latitude: lat,
                longtitude: lon,
                wallet_address: config.wallet_address.clone(),
            };
            let payload = MqttPayload { machine, drone };
            let bytes = bincode::serialize(&payload).unwrap();
            let _ = app.send_payload(bytes).await;
        }
    })
    .await?;

    Ok(())
}

fn generate_random_point(lat: f64, lon: f64, max_distance_km: f64) -> (f64, f64) {
    let mut rng = rand::thread_rng();

    // Convert latitude and longitude from degrees to radians
    let lat_rad = lat.to_radians();
    let lon_rad = lon.to_radians();

    // Generate two random numbers
    let u: f64 = rng.gen(); // Uniform random number between 0 and 1
    let v: f64 = rng.gen();

    // Random distance with uniform distribution over the area
    let distance = max_distance_km * u.sqrt();
    let bearing = 2.0 * PI * v;

    // Convert distance to angular distance in radians
    let angular_distance = distance / EARTH_RADIUS_KM;

    // Compute the new latitude
    let sin_lat_rad = lat_rad.sin();
    let cos_lat_rad = lat_rad.cos();
    let sin_angular_distance = angular_distance.sin();
    let cos_angular_distance = angular_distance.cos();

    let sin_new_lat_rad =
        sin_lat_rad * cos_angular_distance + cos_lat_rad * sin_angular_distance * bearing.cos();
    let new_lat_rad = sin_new_lat_rad.asin();

    // Compute the new longitude
    let y = bearing.sin() * sin_angular_distance * cos_lat_rad;
    let x = cos_angular_distance - sin_lat_rad * new_lat_rad.sin();
    let new_lon_rad = lon_rad + y.atan2(x);

    // Normalize longitude to be between -π and π
    let new_lon_rad = ((new_lon_rad + 3.0 * PI) % (2.0 * PI)) - PI;

    // Convert the new latitude and longitude from radians to degrees
    let new_lat = new_lat_rad.to_degrees();
    let new_lon = new_lon_rad.to_degrees();

    (new_lat, new_lon)
}

/// Function to get the MAC address of the system
fn get_mac_as_string() -> Option<String> {
    if let Ok(Some(mac)) = get_mac_address() {
        Some(mac.to_string())
    } else {
        None
    }
}

/// Function to generate a UUID v5 based on the MAC address
fn generate_uuid_v5_from_mac(mac: &str) -> Uuid {
    Uuid::new_v5(&NAMESPACE_UUID, mac.as_bytes())
}

fn should_send_payload() -> bool {
    // use sha256 and "mine" for a hash that starts with 0000
    let mut hasher = Sha256::new();
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    hasher.update(random_bytes);
    let result = hasher.finalize();
    let result = result.as_slice();

    // contains 00 at the start
    result[0] == 0 && result[1] == 0 && result[2] == 0
}
