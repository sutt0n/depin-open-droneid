use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use pcap::{Capture, Device, Linktype};
use sqlx::{Pool, Postgres};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    drone::{Drone, DroneBuilder},
    odid::{
        parse_basic_id, parse_location, parse_operator_id, parse_system_message, RemoteIdMessage,
    },
    web::{insert_drone, update_drone, DroneDto, DroneUpdate},
    wifi::{
        enable_monitor_mode, parse_action_frame, parse_open_drone_id_message_pack,
        parse_service_descriptor_attribute, remove_radiotap_header, WifiInterface,
        WifiInterfaceBuilder, WifiOpenDroneIDMessagePack, WIFI_ALLIANCE_OUI,
    },
};
use tokio::sync::broadcast::Sender;

pub async fn start_wifi_task(
    wifi_card: String,
    db_pool: Arc<Mutex<Pool<Postgres>>>,
    drones: Arc<Mutex<HashMap<String, Drone>>>,
    tx: Arc<Mutex<Sender<DroneUpdate>>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut drones = drones.lock().await;
        let wifi_card = wifi_card.as_str();

        if let Err(e) = enable_monitor_mode(wifi_card) {
            eprintln!("Error: {}", e);
            return;
        }

        println!("Using device: {}", wifi_card);

        let mut cap = Capture::from_device(wifi_card)
            .unwrap()
            .promisc(true)
            .immediate_mode(true)
            .open();

        if let Err(e) = cap {
            eprintln!("error opening device \"{}\": {}", wifi_card, e);
            let devices = Device::list().unwrap();
            let device_names = devices
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<String>>();

            eprintln!("available devices: {:?}", device_names);
            return;
        }

        cap.as_mut()
            .unwrap()
            .set_datalink(Linktype::IEEE802_11_RADIOTAP)
            .unwrap();

        let mut wifi_interface: WifiInterface = WifiInterfaceBuilder::default()
            .channel(6)
            .name(wifi_card.to_string())
            .build()
            .unwrap();

        while let Ok(packet) = cap.as_mut().unwrap().next_packet() {
            let data = packet.data;

            if wifi_interface.should_change_channel() {
                wifi_interface.adjust_channel();
            }

            let payload = remove_radiotap_header(data);

            let odid_message_pack: WifiOpenDroneIDMessagePack = match parse_action_frame(payload) {
                Ok((_, frame)) => {
                    if frame.oui != WIFI_ALLIANCE_OUI {
                        continue;
                    }
                    match parse_service_descriptor_attribute(frame.body) {
                        Ok((_, service_descriptor_attribute)) => {
                            match parse_open_drone_id_message_pack(
                                service_descriptor_attribute.service_info,
                            ) {
                                Ok((_, open_drone_id_message_pack)) => open_drone_id_message_pack,
                                Err(e) => {
                                    eprintln!(
                                        "Failed to parse Open Drone ID message pack: {:?}",
                                        e
                                    );
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse service descriptor attribute: {:?}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse IEEE 802.11 action frame: {:?}", e);
                    continue;
                }
            };

            let current_timestamp: DateTime<Utc> = Utc::now();
            wifi_interface.update_last_odid_received(current_timestamp);

            let mut drone: Drone = DroneBuilder::default().build().unwrap();

            for message in odid_message_pack.messages {
                match RemoteIdMessage::from(message.message_type) {
                    RemoteIdMessage::SystemMessage => {
                        if let Ok((_, system_message)) = parse_system_message(&message.message_body)
                        {
                            drone.update_system_message(system_message);
                        }
                    }
                    RemoteIdMessage::BasicId => {
                        if let Ok((_, basic_id_message)) = parse_basic_id(&message.message_body) {
                            drone.update_basic_id(basic_id_message);
                        }
                    }
                    RemoteIdMessage::Location => {
                        if let Ok((_, location_message)) = parse_location(&message.message_body) {
                            drone.update_location(location_message);
                        }
                    }
                    RemoteIdMessage::OperatorId => {
                        if let Ok((_, operator_id_message)) =
                            parse_operator_id(&message.message_body)
                        {
                            drone.update_operator(operator_id_message);
                        }
                    }
                    m => {
                        println!("Unknown message type: {:?} {:?}", message.message_type, m);
                        continue;
                    }
                }
            }

            println!("Checking payload for drone {:?}", drone);

            let drone_id = if let Some(id) = drone.basic_id.as_ref() {
                id.uas_id.clone()
            } else {
                continue;
            };

            if drones.contains_key(&drone_id) {
                let drone = drones.get_mut(&drone_id).unwrap();

                if let Some(last_location) = drone.last_location.clone() {
                    drone.update_location(last_location);
                }

                if let Some(system_message) = drone.system_message.clone() {
                    drone.update_system_message(system_message);
                }

                if let Some(operator) = drone.operator.clone() {
                    drone.update_operator(operator);
                }

                if let Some(basic_id) = drone.basic_id.clone() {
                    drone.update_basic_id(basic_id);
                }
            }

            if drone.payload_ready() && !drone.is_in_db {
                println!("Payload ready for drone");

                println!("Drone ID: {}", drone_id);

                drones.insert(drone_id, drone.clone());

                let drone_dto = DroneDto::from(drone.clone());

                let db_pool = db_pool.lock().await;
                let tx = tx.lock().await;

                println!("Inserting drone into database");

                let drone_dto = insert_drone(drone_dto, &db_pool, &tx).await;

                drone.set_in_db(true, drone_dto.id);
            } else if drone.payload_ready() && drone.is_in_db {
                let drone_dto = DroneDto::from(drone.clone());

                let db_pool = db_pool.lock().await;
                let tx = tx.lock().await;

                update_drone(drone_dto, &db_pool, &tx).await;
            } else {
                drones.insert(drone_id, drone);
            }
        }
    })
}
