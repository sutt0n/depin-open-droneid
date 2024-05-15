use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use tokio::time::Duration;

mod parsers;
mod messages;

use crate::parsers::{parse_basic_id, parse_location};

#[tokio::main]
async fn main() {
    // Create a new manager
    let manager = Manager::new().await.unwrap();

    // Get the first adapter
    let adapters = manager.adapters().await.unwrap();
    let central = adapters.into_iter().nth(0).unwrap();

    // Start scanning for devices
    central.start_scan(ScanFilter::default()).await.unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Continuously read BLE advertisements
    let mut events = central.events().await.unwrap();
    while let Some(event) = events.next().await {
        match event {
            btleplug::api::CentralEvent::DeviceDiscovered(id) => {
                let peripheral = central.peripheral(&id).await.unwrap();
                let properties = peripheral.properties().await.unwrap();
                if let Some(properties) = properties {
                    println!(
                        "Discovered device: {:?}, RSSI: {:?}, Address: {:?}",
                        properties.local_name, properties.rssi, peripheral.address()
                    );
                }
            }
            btleplug::api::CentralEvent::ManufacturerDataAdvertisement { id, manufacturer_data } => {
                let peripheral = central.peripheral(&id).await.unwrap();
                let address = peripheral.address();
                println!("Manufacturer data from {:?}: {:?}", address, manufacturer_data);
                // manufacturer_data to readable bytes
                // get the firs tvalue of the manufacturer data hashmap
                let manufacturer_data = manufacturer_data.values().next().unwrap();
                let data = manufacturer_data.to_vec();
                let data = data.as_slice();

                println!("Data length: {}", data.len());

                // make sure the data is long enough to be a Remote ID message
                if data.len() < 25 {
                    continue;
                }

                // if the first byte is 0x01, it's a basic ID message
                //

                match data[0] {
                    0x0 => {
                        let basic_id = parse_basic_id(data);

                        println!("Basic ID: {:?}", basic_id);
                    }
                    0x1 => {
                        let location = parse_location(data);

                        println!("Location: {:?}", location);
                    }
                    _ => {
                        println!("Unknown message type");
                    }
                }
                
                // parse the data bytes into a readable format
            }
            _ => {}
        }
    }
}
