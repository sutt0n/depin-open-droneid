use std::{collections::HashMap, sync::Arc};

use bluez_async::{BluetoothSession, DeviceId, DiscoveryFilter};
use sqlx::{Pool, Postgres};
use tokio::sync::broadcast::Sender;
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_stream::StreamExt;

use super::parse_bluetooth_advertisement_frame;
use crate::{
    drone::{Drone, DroneBuilder},
    odid::{
        parse_basic_id, parse_location, parse_message_type, parse_operator_id,
        parse_system_message, RemoteIdMessage,
    },
    web::{insert_drone, DroneDto, DroneUpdate},
};

pub type MessageType = u8;

pub async fn start_bluetooth_task(
    device_name: String,
    drones: Arc<Mutex<HashMap<String, Drone>>>,
    db_pool: Arc<Mutex<Pool<Postgres>>>,
    tx: Arc<Mutex<Sender<DroneUpdate>>>,
) -> anyhow::Result<()> {
    println!("Inside async move for BT");

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
        println!("events!");
        let mut drones = drones.lock().await;
        if let Some((device_id, message_type)) =
            handle_bluetooth_event(&mut drones, device_name.as_str(), event).await
        {
            let device_id = device_id.to_string();
            let drone = drones.get_mut(&device_id);

            if drone.is_some() {
                let drone = drone.unwrap();
                if drone.payload_ready() {
                    let drone_dto = DroneDto::from(drone.clone());

                    let db_pool = db_pool.lock().await;
                    let tx = tx.lock().await;

                    if !drone.is_in_db {
                        let inserted_drone = insert_drone(drone_dto, &db_pool, &tx).await;
                        drone.set_in_db(true, inserted_drone.id);
                    } else {
                        #[allow(clippy::collapsible_else_if)]
                        // keeping this so we don't have to fight the borrow checker
                        if message_type == 2 || message_type == 4 {
                            insert_drone(drone_dto, &db_pool, &tx).await;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn handle_bluetooth_event(
    drones: &mut HashMap<String, Drone>,
    device_name: &str,
    event: bluez_async::BluetoothEvent,
) -> Option<(DeviceId, MessageType)> {
    match event {
        bluez_async::BluetoothEvent::Device { id, event } => {
            if !id.to_string().contains(device_name) {
                return Some((id, 69));
            }
            match event {
                bluez_async::DeviceEvent::ServiceData { service_data } => {
                    let data = service_data.values().next().unwrap().as_slice();

                    if data.len() < 20 {
                        return Some((id, 69));
                    }

                    println!("[nom-nom] Bluetooth event.");

                    let drone = drones.get(&id.to_string());

                    if drone.is_none() {
                        let drone = DroneBuilder::default().build().unwrap();
                        drones.insert(id.to_string(), drone);
                    }

                    if let Ok((_, bt_advertisement_frame)) =
                        parse_bluetooth_advertisement_frame(data)
                    {
                        match parse_message_type(&bt_advertisement_frame.message) {
                            Ok((_, message_type)) => match message_type {
                                RemoteIdMessage::SystemMessage => {
                                    if let Ok((_, system_message)) =
                                        parse_system_message(&bt_advertisement_frame.message)
                                    {
                                        let drone = drones.get_mut(&id.to_string()).unwrap();

                                        drone.update_system_message(system_message);
                                    }
                                }
                                RemoteIdMessage::BasicId => {
                                    if let Ok((_, basic_id)) =
                                        parse_basic_id(&bt_advertisement_frame.message)
                                    {
                                        let drone = drones.get_mut(&id.to_string()).unwrap();
                                        drone.update_basic_id(basic_id);
                                    }
                                }
                                RemoteIdMessage::Location => {
                                    if let Ok((_, location)) =
                                        parse_location(&bt_advertisement_frame.message)
                                    {
                                        let drone = drones.get_mut(&id.to_string()).unwrap();
                                        drone.update_location(location);
                                    }
                                }
                                RemoteIdMessage::OperatorId => {
                                    if let Ok((_, operator)) =
                                        parse_operator_id(&bt_advertisement_frame.message)
                                    {
                                        let drone = drones.get_mut(&id.to_string()).unwrap();
                                        drone.update_operator(operator);
                                    }
                                }
                                _ => {
                                    return Some((id, 69));
                                }
                            },
                            Err(_) => {
                                return Some((id, 69));
                            }
                        }
                    }

                    Some((id, 69))
                }

                _ => Some((id, 69)),
            }
        }
        _ => None,
    }
}
