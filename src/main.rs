use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use serde::Serialize;
use std::collections::HashMap;
use tokio::time::Duration;

#[derive(Serialize)]
struct DeviceProperties {
    local_name: Option<String>,
    rssi: Option<i16>,
    address: String,
    manufacturer_data: Option<HashMap<u16, String>>, // Decode as String
}

fn decode_manufacturer_data(data: HashMap<u16, Vec<u8>>) -> HashMap<u16, String> {
    data.into_iter()
        .map(|(key, value)| {
            let decoded = String::from_utf8_lossy(&value).to_string();
            (key, decoded)
        })
        .collect()
}

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
            btleplug::api::CentralEvent::DeviceDiscovered(id)
            | btleplug::api::CentralEvent::ManufacturerDataAdvertisement { id, .. } => {
                let peripheral = central.peripheral(&id).await.unwrap();
                let properties = peripheral.properties().await.unwrap();
                if let Some(properties) = properties {
                    let decoded_manufacturer_data = if !properties.manufacturer_data.is_empty() {
                        Some(decode_manufacturer_data(properties.manufacturer_data.clone()))
                    } else {
                        None
                    };
                    let device_props = DeviceProperties {
                        local_name: properties.local_name,
                        rssi: properties.rssi,
                        address: peripheral.address().to_string(),
                        manufacturer_data: decoded_manufacturer_data,
                    };
                    let json = serde_json::to_string(&device_props).unwrap();
                    println!("{}", json);
                }
            }
            _ => {}
        }
    }
}
