use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use log::{debug, error, info, trace, warn};
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
        enable_monitor_mode, is_action_frame, is_beacon_frame, parse_action_frame,
        parse_beacon_frame, parse_open_drone_id_message_pack, parse_service_descriptor_attribute,
        remove_radiotap_header, WifiOpenDroneIDMessagePack,
    },
};
use tokio::sync::broadcast::Sender;

use super::WifiInterface;

pub async fn start_wifi_task(
    wifi_card: String,
    db_pool: Arc<Mutex<Pool<Postgres>>>,
    drones: Arc<Mutex<HashMap<String, Drone>>>,
    tx: Arc<Mutex<Sender<DroneUpdate>>>,
    wifi_interface: Arc<Mutex<WifiInterface>>,
) -> anyhow::Result<()> {
    let wifi_card = wifi_card.as_str().to_owned();

    if let Err(e) = enable_monitor_mode(&wifi_card) {
        eprintln!("Error: {}", e);
        return Ok(());
    }

    println!("Using device: {}", wifi_card);

    let mut cap = Capture::from_device(&*wifi_card)
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
        return Ok(());
    }

    cap.as_mut()
        .unwrap()
        .set_datalink(Linktype::IEEE802_11_RADIOTAP)
        .unwrap();

    while let Ok(packet) = cap.as_mut().unwrap().next_packet() {
        tokio::task::yield_now().await;

        debug!("Checking packet {:?}", packet.header.len);

        let data = packet.data;

        if String::from_utf8_lossy(&data).contains("DroneBeacon") {
            debug!("DroneBeacon found {:?}", data);
        }

        let payload: Option<&[u8]> = remove_radiotap_header(data).await;

        if payload.is_none() {
            continue;
        }

        let payload = payload.unwrap();

        if is_beacon_frame(payload, 0).await {
            trace!("Beacon frame found");
        }

        if is_action_frame(payload, 0).await {
            trace!("Action frame found");
        }

        let odid_message_pack: Option<WifiOpenDroneIDMessagePack> = if is_action_frame(payload, 0)
            .await
        {
            match parse_action_frame(payload).await {
                Ok((_, frame)) => match parse_service_descriptor_attribute(frame.body).await {
                    Ok((_, service_descriptor_attribute)) => {
                        match parse_open_drone_id_message_pack(
                            service_descriptor_attribute.service_info,
                        )
                        .await
                        {
                            Ok((_, open_drone_id_message_pack)) => Some(open_drone_id_message_pack),
                            Err(e) => {
                                debug!(
                                        "[action frame] Failed to parse Open Drone ID message pack: {:?}",
                                        e
                                    );
                                debug!("data: {:?}", data);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        trace!("Failed to parse service descriptor attribute: {:?}", e);
                        None
                    }
                },
                Err(e) => {
                    trace!("Failed action frame: {:?}", e);
                    None
                }
            }
        } else if is_beacon_frame(payload, 0).await {
            match parse_beacon_frame(payload).await {
                Ok((_, beacon_frame)) => {
                    match parse_open_drone_id_message_pack(beacon_frame.vendor_specific_data).await
                    {
                        Ok((_, open_drone_id_message_pack)) => Some(open_drone_id_message_pack),
                        Err(e) => {
                            debug!(
                                "[beacon frame] Failed to parse Open Drone ID message pack: {:?}",
                                e
                            );
                            debug!("data: {:?}", data);
                            None
                        }
                    }
                }
                Err(e) => {
                    trace!("Failed to parse Beacon/Action frames: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        if odid_message_pack.is_none() {
            continue;
        }

        let odid_message_pack = odid_message_pack.unwrap();

        if odid_message_pack.messages.len() > 0 {
            println!("Received ODID message pack {:?}", odid_message_pack);
        } else {
            continue;
        }

        let current_timestamp: DateTime<Utc> = Utc::now();

        {
            let mut wifi_interface = wifi_interface.lock().await;
            wifi_interface.update_last_odid_received(current_timestamp);
        }

        let mut drone: Drone = DroneBuilder::default().build().unwrap();

        for message in odid_message_pack.messages {
            match RemoteIdMessage::from(message.message_type) {
                RemoteIdMessage::SystemMessage => {
                    if let Ok((_, system_message)) = parse_system_message(&message.message_body) {
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
                    if let Ok((_, operator_id_message)) = parse_operator_id(&message.message_body) {
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

        {
            let mut drones = drones.lock().await;

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
        }

        if drone.payload_ready() && !drone.is_in_db {
            println!("Payload ready for drone");

            println!("Drone ID: {}", drone_id);

            {
                let mut drones = drones.lock().await;
                drones.insert(drone_id.clone(), drone.clone());
            }

            let drone_dto = DroneDto::from(drone.clone());

            let (db_pool, tx) = {
                let db_pool = db_pool.lock().await.clone();
                let tx = tx.lock().await.clone();
                (db_pool, tx)
            };

            println!("Inserting drone into database");

            let drone_dto = insert_drone(drone_dto, &db_pool, &tx).await;

            let mut drones = drones.lock().await;
            if let Some(mut drone) = drones.get_mut(&drone_id) {
                drone.set_in_db(true, drone_dto.id);
            }
        } else if drone.payload_ready() && drone.is_in_db {
            let drone_dto = DroneDto::from(drone.clone());

            let (db_pool, tx) = {
                let db_pool = db_pool.lock().await.clone();
                let tx = tx.lock().await.clone();
                (db_pool, tx)
            };

            update_drone(drone_dto, &db_pool, &tx).await;
        } else {
            let mut drones = drones.lock().await;
            drones.insert(drone_id, drone);
        }

        // Yield control to allow other tasks to run
        tokio::task::yield_now().await;
    }

    Ok(())
}
