use std::collections::HashMap;

use bluez_async::DeviceId;

use crate::{
    drone::Drone,
    parsers::{parse_basic_id, parse_location, parse_operator_id, parse_system_message},
};

pub async fn handle_bluetooth_event(
    drones: &mut HashMap<DeviceId, Drone>,
    device_name: &str,
    event: bluez_async::BluetoothEvent,
) -> Option<DeviceId> {
    match event {
        bluez_async::BluetoothEvent::Device { id, event } => {
            if !id.to_string().contains(device_name) {
                return Some(id);
            }
            match event {
                bluez_async::DeviceEvent::ServiceData { service_data } => {
                    let data = service_data.values().next().unwrap().as_slice();

                    if data.len() < 20 {
                        return Some(id);
                    }

                    let drone = drones.get(&id);

                    if drone.is_none() {
                        let drone = Drone::new(None, None, None, None);
                        drones.insert(id.clone(), drone);
                    }

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
                                    let basic_id = parse_basic_id(data);
                                    drones.get_mut(&id).unwrap().update_basic_id(basic_id);
                                }
                                1 => {
                                    let location = parse_location(data);
                                    drones.get_mut(&id).unwrap().update_location(location);
                                }
                                2 => {
                                    println!("Auth message");
                                }
                                4 => {
                                    let system_message = parse_system_message(data);
                                    drones
                                        .get_mut(&id)
                                        .unwrap()
                                        .update_system_message(system_message);
                                }
                                5 => {
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
                        }
                        _ => {}
                    }

                    Some(id)
                }

                _ => Some(id),
            }
        }
        _ => None,
    }
}
