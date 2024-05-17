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
                    println!("Device: {:?} {:?}",id,event);
                    match event {
                        bluez_async::DeviceEvent::ServiceData { service_data } => {
                            // get first value of service data
                            let first_value = service_data.values().next().unwrap();
                            // to &[u8]
                            let data = first_value.as_slice();

                            if data.len() < 24 {
                                continue;
                            }

                            println!("Service Data: {:?}", service_data);

                        }
                        _ => {}
                    }
                }
                bluez_async::BluetoothEvent::Adapter { id, event } => todo!(),
                bluez_async::BluetoothEvent::Characteristic { id, event } => todo!(), }
        }
    }).await?;

    Ok(())
}
