use std::collections::HashMap;

use bluez_async::DeviceId;

use crate::{
    drone::Drone,
    parsers::{parse_basic_id, parse_location, parse_operator_id, parse_system_message},
};

pub type MessageType = u8;

pub async fn handle_bluetooth_event(
    drones: &mut HashMap<DeviceId, Drone>,
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

                    let drone = drones.get(&id);

                    if drone.is_none() {
                        let drone = Drone::new(None, None, None, None);
                        drones.insert(id.clone(), drone);
                    }

                    let clone_data = data.clone();

                    match data[0] {
                        0x0D => {
                            // skip first two bytes
                            let data = &data[2..];

                            let header = data[0];
                            // message type is 4 bits, protocol version is last 4 bits
                            let message_type = (header & 0xF0) >> 4;
                            let _protocol_version = header & 0x0F;

                            let data = &data[1..];

                            match message_type {
                                0 => {
                                    println!("Basic ID {:?}", clone_data);
                                    let basic_id = parse_basic_id(data);
                                    drones.get_mut(&id).unwrap().update_basic_id(basic_id);
                                }
                                1 => {
                                    println!("Location {:?}", clone_data);
                                    let location = parse_location(data);
                                    drones.get_mut(&id).unwrap().update_location(location);
                                }
                                3 => {
                                    println!("Self ID {:?}", clone_data);
                                    println!("Self ID message");
                                }
                                2 => {
                                    println!("Self ID {:?}", clone_data);
                                    println!("Auth message");
                                }
                                4 => {
                                    println!("System Message {:?}", clone_data);
                                    let system_message = parse_system_message(data);
                                    drones
                                        .get_mut(&id)
                                        .unwrap()
                                        .update_system_message(system_message);
                                }
                                5 => {
                                    println!("Operator ID {:?}", clone_data);
                                    let operator = parse_operator_id(data);
                                    drones.get_mut(&id).unwrap().update_operator(operator);
                                }
                                0xF => {
                                    println!("Message Pack");
                                }
                                message => {
                                    println!("Unknown message type {}", message);
                                }
                            }

                            return Some((id, message_type));
                        }
                        _ => {}
                    }

                    Some((id, 69))
                }

                _ => Some((id, 69)),
            }
        }
        _ => None,
    }
}
