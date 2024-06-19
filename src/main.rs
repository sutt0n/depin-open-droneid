use std::collections::HashMap;
use std::net::SocketAddr;

use axum::Router;
use bluez_async::{BluetoothSession, DeviceId, DiscoveryFilter};
use futures::stream::StreamExt;
use models::DroneDto;
use pcap::{Capture, Device};
use routes::insert_drone;

mod bluetooth;
mod drone;
mod errors;
mod messages;
mod models;
mod parsers;
mod router;
mod routes;
mod templates;
mod wifi;

use crate::wifi::{disable_monitor_mode, enable_monitor_mode};
use crate::bluetooth::handle_bluetooth_event;
use crate::drone::Drone;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device_name = "hci0";
    let wifi_device_name = "wlx98254aed7e95";

    let mut drones: HashMap<DeviceId, Drone> = HashMap::new();

    let conn_url = std::env::var("DATABASE_URL")
        .expect("Env var DATABASE_URL is required for this example.");

    let sqlx_connection = sqlx::PgPool::connect(&conn_url).await.unwrap();

    // run the migrations
    sqlx::migrate!()
        .run(&sqlx_connection)
        .await
        .expect("Failed to run migrations");

    let (router, tx) = router::init_router(sqlx_connection.clone());


    let wifi_task = tokio::spawn(async move {
        if let Err(e) = enable_monitor_mode(wifi_device_name) {
            eprintln!("Error: {}", e);
            return;
        }

        println!("Using device: {}", wifi_device_name);

        let mut cap = Capture::from_device(wifi_device_name.as_str()).unwrap()
            .promisc(true)
            .open();

        if let Err(e) = cap {
            eprintln!("error opening device \"{}\": {}", wifi_device_name, e);
            let devices = Device::list().unwrap();
            let device_names = devices.iter().map(|d| d.name.clone()).collect::<Vec<String>>();

            eprintln!("available devices: {:?}", device_names);
            return;
        }

        let mut cap = cap.unwrap().setnonblock().unwrap();

        cap.for_each(None, |packet| {
            println!("packet {:?}", packet);
        });
    });

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
