use std::collections::HashMap;

use bluez_async::{BluetoothSession, DeviceId, DiscoveryFilter};
use futures::stream::StreamExt;

mod bluetooth;
mod drone;
mod messages;
mod parsers;

use crate::bluetooth::handle_bluetooth_event;
use crate::drone::Drone;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Spawn a task to handle bluetooth events
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            handle_bluetooth_event(&mut drones, device_name, event);
        }
    })
    .await?;

    Ok(())
}
