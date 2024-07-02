use std::collections::HashMap;

use bluez_async::{BluetoothSession, DeviceId, DiscoveryFilter};
use drone::Drone;
use pcap::{Capture, Device, Linktype};

mod bluetooth;
mod odid;
mod drone;
mod web;
mod wifi;

use wifi::{
    enable_monitor_mode, 
    parse_service_descriptor_attribute, 
    remove_radiotap_header, 
    parse_open_drone_id_message_pack, 
    parse_action_frame, 
    WifiOpenDroneIDMessagePack
};
use web::{init_router, start_webserver};
use crate::{bluetooth::handle_bluetooth_event, wifi::WIFI_ALLIANCE_OUI};

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

    let (router, tx) = init_router(sqlx_connection.clone());

    let wifi_task = tokio::spawn(async move {
        let wifi_card: &str = "wlx08beac26e3e8";

        if let Err(e) = enable_monitor_mode(wifi_card) {
            eprintln!("Error: {}", e);
            return;
        }

        println!("Using device: {}", wifi_card);

        let mut cap = Capture::from_device(wifi_card).unwrap()
            .promisc(true)
            .immediate_mode(true)
            .open();

        if let Err(e) = cap {
            eprintln!("error opening device \"{}\": {}", wifi_card, e);
            let devices = Device::list().unwrap();
            let device_names = devices.iter().map(|d| d.name.clone()).collect::<Vec<String>>();

            eprintln!("available devices: {:?}", device_names);
            return;
        }

        cap.as_mut().unwrap().set_datalink(Linktype::IEEE802_11_RADIOTAP).unwrap();

        while let Ok(packet) = cap.as_mut().unwrap().next_packet() {
            let data = packet.data;

            let payload = remove_radiotap_header(data);

            let open_drone_id_message_pack: WifiOpenDroneIDMessagePack = match parse_action_frame(payload) {
                Ok((_, frame)) => {
                    if frame.oui != WIFI_ALLIANCE_OUI {
                        continue;
                    }
                    match parse_service_descriptor_attribute(frame.body) {
                        Ok((_, service_descriptor_attribute)) => {
                            match parse_open_drone_id_message_pack(service_descriptor_attribute.service_info) {
                                Ok((_, open_drone_id_message_pack)) => open_drone_id_message_pack,
                                Err(e) => {
                                    eprintln!("Failed to parse Open Drone ID message pack: {:?}", e);
                                    continue;
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to parse service descriptor attribute: {:?}", e);
                            continue;
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to parse IEEE 802.11 action frame: {:?}", e);
                    continue;
                }
            };

            println!("{:?}", open_drone_id_message_pack);

            // todo: parse open drone id message pack and insert into db
        }
    });

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

