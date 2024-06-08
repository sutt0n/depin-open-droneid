use std::collections::HashMap;

use bluez_async::{BluetoothSession, DeviceId, DiscoveryFilter};
use futures::stream::StreamExt;
use models::DroneDto;
use routes::insert_drone;
use sqlx::postgres::PgPoolOptions;

mod bluetooth;
mod drone;
mod errors;
mod messages;
mod models;
mod parsers;
mod router;
mod routes;

use crate::bluetooth::handle_bluetooth_event;
use crate::drone::Drone;
use crate::routes::update_drone;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device_name = "hci1";

    let mut drones: HashMap<DeviceId, Drone> = HashMap::new();

    let sqlx_connection = PgPoolOptions::new()
        .connect("postgres://postgres:postgres@localhost:5432/db")
        .await
        .unwrap();

    let (_, session) = BluetoothSession::new().await?;
    let mut events = session.event_stream().await?;
    session
        .start_discovery_with_filter(&DiscoveryFilter {
            duplicate_data: Some(true),
            ..DiscoveryFilter::default()
        })
        .await?;

    let (router, tx) = router::init_router(sqlx_connection.clone());

    // Spawn a task to handle bluetooth events
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            if let Some(device_id) = handle_bluetooth_event(&mut drones, device_name, event) {
                let drone = drones.get_mut(&device_id);

                if drone.is_some() {
                    let drone = drone.unwrap();
                    if drone.payload_ready() {
                        let drone_dto = DroneDto::from(drone.clone());

                        if !drone.is_in_db {
                            let inserted_drone =
                                insert_drone(drone_dto, &sqlx_connection, &tx).await;
                            drone.set_in_db(true, inserted_drone.id);
                        } else {
                            update_drone(drone_dto, &sqlx_connection, &tx).await;
                        }
                    }
                }
            }
        }
    })
    .await?;

    // Spawn a task to serve axum
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
            .await
            .unwrap();

        let _ = axum::serve(listener, router);
    })
    .await?;

    Ok(())
}
