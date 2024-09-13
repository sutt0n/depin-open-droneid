use ::chrono::{DateTime, Utc};
use fake::faker::address::en::{Latitude, Longitude};
use fake::faker::boolean::en::*;
use fake::faker::company::en::*;
use fake::faker::company::raw::CompanyName;
use fake::faker::lorem::en::*;
use fake::faker::name::en::*;
use fake::faker::number::en::*;
use fake::locales::EN;
use fake::{Dummy, Fake, Faker};
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
    pub drone: DroneSerialized,
    pub id: i32,
}

#[derive(Debug, Dummy, Clone, sqlx::FromRow, Serialize, Deserialize)]
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

impl DroneDto {
    pub fn dummy() -> Self {
        let id = Faker.fake::<i32>();
        let serial_number = CompanyName(EN).fake();
        let created = Faker.fake::<DateTime<Utc>>();
        let latitude: f64 = Latitude().fake();
        let longitude: f64 = Longitude().fake();
        let altitude = Faker.fake::<f64>() as f64;
        let x_speed = Faker.fake::<f64>() as f64;
        let y_speed = Faker.fake::<f64>() as f64;
        let yaw = Faker.fake::<f64>() as f64;
        let pilot_latitude: f64 = Latitude().fake();
        let pilot_longitude: f64 = Longitude().fake();
        let home_latitude: f64 = Latitude().fake();
        let home_longitude: f64 = Longitude().fake();

        DroneDto {
            id,
            serial_number,
            created,
            latitude,
            longitude,
            altitude,
            x_speed,
            y_speed,
            yaw,
            pilot_latitude,
            pilot_longitude,
            home_latitude,
            home_longitude,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneSerialized {
    pub serial_number: String,
    pub created: DateTime<Utc>,
    pub altitude: f64,
    pub x_speed: f64,
    pub y_speed: f64,
    pub yaw: f64,
    pub position: Position,
    pub pilot_position: Position,
    pub home_position: Position,
}

impl From<DroneDto> for DroneSerialized {
    fn from(drone_dto: DroneDto) -> Self {
        DroneSerialized {
            serial_number: drone_dto.serial_number,
            created: drone_dto.created,
            altitude: drone_dto.altitude,
            x_speed: drone_dto.x_speed,
            y_speed: drone_dto.y_speed,
            yaw: drone_dto.yaw,
            position: Position {
                lat: drone_dto.latitude,
                lng: drone_dto.longitude,
            },
            pilot_position: Position {
                lat: drone_dto.pilot_latitude,
                lng: drone_dto.pilot_longitude,
            },
            home_position: Position {
                lat: drone_dto.home_latitude,
                lng: drone_dto.home_longitude,
            },
        }
    }
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
