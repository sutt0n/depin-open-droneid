use ::chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::drone::Drone;

#[derive(Clone, Serialize, Debug)]
pub enum MutationKind {
    Create,
    Update,
}

#[derive(Clone, Serialize, Debug)]
pub struct DroneUpdate {
    pub mutation_kind: MutationKind,
    pub id: i32,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct DroneDto {
    pub id: i32,
    pub serial_number: String,
    pub created: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub x_speed: f64,
    pub y_speed: f64,
    pub yaw: f64,
    pub pilot_latitude: f64,
    pub pilot_longitude: f64,
    pub home_latitude: f64,
    pub home_longitude: f64,
}

impl From<Drone> for DroneDto {
    fn from(drone: Drone) -> Self {
        let latitude_int = drone.last_location.as_ref().unwrap().latitude_int;
        let longitude_int = drone.last_location.as_ref().unwrap().longitude_int;

        let latitude: f64 = (latitude_int as f64 / 10_f64.powi(7)).into();
        let longitude: f64 = (longitude_int as f64 / 10_f64.powi(7)).into();

        let altitude: f64 = drone.last_location.as_ref().unwrap().height.into();
        let altitude: f64 = (altitude * 0.5) - 1000.0;

        let ew_direction = drone.last_location.as_ref().unwrap().ew_direction;
        let yaw: f64 = drone
            .last_location
            .as_ref()
            .unwrap()
            .tracking_direction
            .into();
        let yaw = if ew_direction == 1 { yaw + 180.0 } else { yaw };

        let speed_multiplier = drone.last_location.as_ref().unwrap().speed_multiplier;
        let speed: f64 = drone.last_location.as_ref().unwrap().speed.into();
        let speed = if speed_multiplier == 1 {
            speed * 0.25
        } else {
            if speed > 0.0 {
                speed * 0.75 + 63.75
            } else {
                speed
            }
        };

        let y_speed: f64 = drone.last_location.as_ref().unwrap().vertical_speed.into();
        let y_speed = y_speed * 0.5;

        let pilot_latitude_int = drone.system_message.as_ref().unwrap().operator_latitude_int;
        let pilot_longitude_int = drone.system_message.unwrap().operator_longitude_int;

        let pilot_latitude: f64 = (pilot_latitude_int as f64 / 10_f64.powi(7)).into();
        let pilot_longitude: f64 = (pilot_longitude_int as f64 / 10_f64.powi(7)).into();

        let drone_first_location = drone.location_history.first().unwrap();

        let home_latitude_int = drone_first_location.latitude_int;
        let home_longitude_int = drone_first_location.longitude_int;

        let home_latitude: f64 = (home_latitude_int as f64 / 10_f64.powi(7)).into();
        let home_longitude: f64 = (home_longitude_int as f64 / 10_f64.powi(7)).into();

        let id = if drone.is_in_db { drone.db_id } else { 0 };

        let created: DateTime<Utc> = Utc::now();

        DroneDto {
            serial_number: drone.basic_id.unwrap().uas_id,
            latitude,
            longitude,
            altitude,
            yaw,
            x_speed: speed,
            y_speed,
            pilot_latitude,
            pilot_longitude,
            home_latitude,
            home_longitude,
            id,
            created,
        }
    }
}
