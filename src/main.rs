use std::collections::HashMap;
use std::net::SocketAddr;

use axum::Router;
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
mod templates;

use crate::bluetooth::handle_bluetooth_event;
use crate::drone::Drone;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device_name = "hci0";

    let mut drones: HashMap<DeviceId, Drone> = HashMap::new();

    let sqlx_connection = PgPoolOptions::new()
        .connect("postgres://difumetti:bRA2R9K5FUwGmHWYxlMQ@postgres.sharkbackup.com:5432/db")
        .await
        .unwrap();

    // run the migrations
    sqlx::migrate!()
        .run(&sqlx_connection)
        .await
        .expect("Failed to run migrations");

    let (router, tx) = router::init_router(sqlx_connection.clone());

    // Spawn a task to handle bluetooth events
    let bt_task = tokio::spawn(async move {
        let (_, session) = BluetoothSession::new().await.unwrap();
        let mut events = session.event_stream().await.unwrap();
        session
            .start_discovery_with_filter(&DiscoveryFilter {
                duplicate_data: Some(true),
                ..DiscoveryFilter::default()
            })
            .await
            .unwrap();

        println!("Scanning for Bluetooth advertisement data.");

        while let Some(event) = events.next().await {
            if let Some((device_id, message_type)) =
                handle_bluetooth_event(&mut drones, device_name, event).await
            {
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
                            if message_type == 2 || message_type == 4 {
                                insert_drone(drone_dto, &sqlx_connection, &tx).await;
                            }
                        }
                    }
                }
            }
        }
    });

    // Run both tasks concurrently
    let (_, _) = tokio::join!(bt_task, start_webserver(router));

    Ok(())
}

async fn start_webserver(router: Router) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("About to serve axum");

    let _ = axum::serve(listener, router).await.unwrap();

    println!("Server running on 3001");
}
