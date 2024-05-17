//! Example to log Bluetooth events, including duplicate manufacturer-specific advertisement data.

use bluez_async::{BluetoothSession, DiscoveryFilter};
use futures::stream::StreamExt;

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

    println!("Events:");
    // while let Some(event) = events.next().await {
    //     println!("{:?}", event);
    // }

    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                bluez_async::BluetoothEvent::Device{id,event} => {
                    match event {
                        bluez_async::DeviceEvent::ServiceData { service_data } => {
                            // get first value of service data
                            let first_value = service_data.values().next().unwrap();
                            // to &[u8]
                            let data = first_value.as_slice();

                            if data.len() < 24 {
                                continue;
                            }

                            // to lossy string
                            let data_str = String::from_utf8_lossy(data);
                            println!("Service Data: {:?} {:?}", data, data_str);

                        }
                        _ => {}
                    }
                }
                bluez_async::BluetoothEvent::Adapter { id, event } => {},
                bluez_async::BluetoothEvent::Characteristic { id, event } => {}, }
        }
    }).await?;

    Ok(())
}
