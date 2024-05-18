//! Example to log Bluetooth events, including duplicate manufacturer-specific advertisement data.

use bluez_async::{BluetoothSession, DiscoveryFilter};
use futures::stream::StreamExt;

mod parsers;
mod messages;

use crate::parsers::{parse_basic_id, parse_location, parse_operator_id, parse_system_message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    let (_, session) = BluetoothSession::new().await?;
    let mut events = session.event_stream().await?;
    session
        .start_discovery_with_filter(&DiscoveryFilter {
            duplicate_data: Some(true),
            ..DiscoveryFilter::default()
        })
        .await?;

    let adapters = session.get_adapters().await?;
    // let mut adapter_to_use = None;

    for adapter in adapters {
        if adapter.powered && adapter.discovering {
            let adapter = session.get_adapter_info(&adapter.id).await?;
            println!("Adapter: {:?}", adapter);
        }
    }

    println!("Events:");
    // while let Some(event) = events.next().await {
    //     println!("{:?}", event);
    // }

    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                bluez_async::BluetoothEvent::Device{id,event} => {
                    println!("Device Event ID: {:?}", id);
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

                                            println!("Basic ID: {:?}", basic_id);
                                        }
                                        1 => {
                                            println!("Location!");
                                            let location = parse_location(data);
                                            println!("Location: {:?}", location);
                                        }
                                        4 => {
                                            let system_message = parse_system_message(data);
                                            println!("System Message: {:?}", system_message);

                                        }
                                        5 => {
                                            let operator = parse_operator_id(data);
                                            println!("Operator: {:?}", operator);
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
