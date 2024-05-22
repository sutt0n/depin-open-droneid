use std::collections::HashMap;

use bluez_async::{BluetoothSession, DiscoveryFilter, DeviceId};
use futures::stream::StreamExt;

mod parsers;
mod messages;
mod drone;

use crate::parsers::{parse_basic_id, parse_location, parse_operator_id, parse_system_message};
use crate::drone::Drone;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let device_name = "hci1";

    let mut drones: HashMap<DeviceId, Drone> = HashMap::new();

    let (_, session) = BluetoothSession::new().await?;
    let mut events = session.event_stream().await?;
    session
        .start_discovery_with_filter(&DiscoveryFilter {
            duplicate_data: Some(true),
            ..DiscoveryFilter::default()
        })
        .await?;

    println!("Events:");
    // while let Some(event) = events.next().await {
    //     println!("{:?}", event);
    // }

    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                bluez_async::BluetoothEvent::Device{id,event} => {
                    if !id.to_string().contains(device_name) {
                        continue;
                    }
                    match event {
                        bluez_async::DeviceEvent::ServiceData { service_data } => {
                            // get first value of service data
                            let first_value = service_data.values().next().unwrap();
                            // to &[u8]
                            let data = first_value.as_slice();

                            if data.len() < 20 {
                                continue;
                            }

                            // to lossy string
                            let data_str = String::from_utf8_lossy(data);
                            println!("Service Data: {} {:?} {:?}", data[0], data, data_str);

                            // check if drone is already in hashmap
                            let drone = drones.get(&id);

                            if drone.is_none() {
                                let drone = Drone::new(None, None, None, None);
                                drones.insert(id.clone(), drone);
                            } else {
                                if drone.unwrap().payload_ready() {
                                    println!("Payload Ready {:?}", drone);
                                }
                            }

                            match data[0] {
                                0x0D => {
                                    // skip first two bytes
                                    let data = &data[2..];

                                    let header = data[0];
                                    // message type is 4 bits, protocol version is last 4 bits
                                    let message_type = (header & 0xF0) >> 4;
                                    let protocol_version = header & 0x0F;

                                    let data = &data[1..];

                                    println!("Message Type: {} Protocol Version: {}", message_type, protocol_version);

                                    match message_type {
                                        0 => {
                                            let basic_id = parse_basic_id(data);
                                            drones.get_mut(&id).unwrap().update_basic_id(basic_id);

                                        }
                                        1 => {
                                            let location = parse_location(data);
                                            drones.get_mut(&id).unwrap().update_location(location);
                                        }
                                        4 => {
                                            let system_message = parse_system_message(data);
                                            drones.get_mut(&id).unwrap().update_system_message(system_message);

                                        }
                                        5 => {
                                            let operator = parse_operator_id(data);
                                            drones.get_mut(&id).unwrap().update_operator(operator);
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }

                        }

                        _ => {}
                    }
                }
                bluez_async::BluetoothEvent::Adapter { id, event } => {},
                bluez_async::BluetoothEvent::Characteristic { id, event } => {
                }, }
        }
    }).await?;

    Ok(())
}
