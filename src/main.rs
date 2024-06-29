use std::collections::HashMap;
use std::net::SocketAddr;

use axum::Router;
use bluez_async::{BluetoothSession, DeviceId, DiscoveryFilter};
use futures::stream::StreamExt;
use models::DroneDto;
use pcap::{Capture, Device};
use routes::insert_drone;
use pnet::datalink::{ self, NetworkInterface, Channel::Ethernet};
use simple_wifi::*;
use std::process;

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
    let wifi_device_name = "wlan0mon";

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
        let wifi_card: &str = "enp1s0";
        
            // Finding the interface that matches the wifi_card. So it can be used for sniffing.
            let interface: NetworkInterface = datalink::interfaces()
                .into_iter()
                .filter(|iface: &NetworkInterface| iface.name == wifi_card)
                .next()
                .unwrap();
        
            // Setting up the channel to the interface so you can sniff Wi-Fi packets.
            let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
                Ok(Ethernet(tx, rx)) => (tx, rx),
                Ok(_) => {
                    println!("\x1b[38;5;9mUnhandled channel type\x1b[0m");
                    process::exit(0);
                },
                Err(e) => {
                    println!("\x1b[38;5;9mAn error occurred when creating the datalink channel: {e}\x1b[0m");
                    process::exit(0);
                }
        
            };

        loop {
            let pkt: &[u8] = match rx.next() {
                Ok(pkt) => pkt,
                Err(_) => continue
            };

            let pkt_info: Packet = match Packet::new(pkt) {
                Ok(t) => t,
                Err(e) => {
                    println!("\x1b[38;5;9m{e}\x1b[0m");
                    let _ = write_pcap(&pkt.to_vec(), "/home/user/error.pcap");
                    continue;
                }
            };

            println!("{pkt_info:#?}");
            println!(
                "Address1: {}, Address2: {}\nAddress3: {}, Address4: {}",
                pkt_info.addr1, pkt_info.addr2, pkt_info.addr3, pkt_info.addr4
            )
        }
    });
    //
    // Spawn a task to handle bluetooth events
    // let bt_task = tokio::spawn(async move {
    //     let (_, session) = BluetoothSession::new().await.unwrap();
    //     let mut events = session.event_stream().await.unwrap();
    //     session
    //         .start_discovery_with_filter(&DiscoveryFilter {
    //             duplicate_data: Some(true),
    //             ..DiscoveryFilter::default()
    //         })
    //         .await
    //         .unwrap();
    //
    //     println!("Scanning for Bluetooth advertisement data.");
    //
    //     while let Some(event) = events.next().await {
    //         if let Some((device_id, message_type)) =
    //             handle_bluetooth_event(&mut drones, device_name, event).await
    //         {
    //             let drone = drones.get_mut(&device_id);
    //
    //             if drone.is_some() {
    //                 let drone = drone.unwrap();
    //                 if drone.payload_ready() {
    //                     let drone_dto = DroneDto::from(drone.clone());
    //
    //                     if !drone.is_in_db {
    //                         let inserted_drone =
    //                             insert_drone(drone_dto, &sqlx_connection, &tx).await;
    //                         drone.set_in_db(true, inserted_drone.id);
    //                     } else {
    //                         if message_type == 2 || message_type == 4 {
    //                             insert_drone(drone_dto, &sqlx_connection, &tx).await;
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // });

    // Run both tasks concurrently
    // let (_, _) = tokio::join!(bt_task, start_webserver(router));

    let _ = tokio::join!(wifi_task);

    Ok(())
}

async fn start_webserver(router: Router) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("About to serve axum");

    let _ = axum::serve(listener, router).await.unwrap();

    println!("Server running on 3001");
}
