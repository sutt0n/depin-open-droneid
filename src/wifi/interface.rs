use std::process::Command;

use chrono::{DateTime, Utc};
use derive_builder::Builder;

#[derive(Debug, Builder, Clone)]
pub struct WifiInterface {
    #[builder(default = "String::from(\"wlan0\")")]
    pub name: String,
    #[builder(default = "6")]
    pub channel: u8,
    #[builder(default = "None")]
    pub last_odid_received: Option<DateTime<Utc>>,
}

impl WifiInterface {
    // Time in seconds to change the channel
    const TIME_TO_CHANGE_CHANNEL: i64 = 10;

    pub fn update_last_odid_received(&mut self, timestamp: DateTime<Utc>) {
        self.last_odid_received = Some(timestamp);
    }

    pub fn should_change_channel(&self) -> bool {
        let current_time = Utc::now();

        match self.last_odid_received {
            Some(last_odid_received) => {
                let time_diff = current_time
                    .signed_duration_since(last_odid_received)
                    .num_seconds();
                time_diff > WifiInterface::TIME_TO_CHANGE_CHANNEL
            }
            None => true,
        }
    }

    pub fn adjust_channel(&mut self) {
        match self.channel {
            1 => self.channel = 6,
            6 => self.channel = 11,
            11 => self.channel = 1,
            _ => self.channel = 6,
        }

        println!("Adjusting channel to {}", self.channel);

        self.last_odid_received = None;

        // Change adjust_channel
        let _ = Command::new("iwconfig")
            .args([
                self.name.as_str(),
                "channel",
                self.channel.to_string().as_str(),
            ])
            .output()
            .expect("failed to execute process");
    }
}

pub fn enable_monitor_mode(device: &str) -> Result<(), String> {
    // Check if the device is already in monitoring mode
    let check_mode = Command::new("iwconfig")
        .arg(device)
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8_lossy(&check_mode.stdout);

    if !output.contains("Mode:Monitor") {
        // Enable monitoring mode using airmon-ng
        let start_mon = Command::new("sudo")
            .args(["airmon-ng", "start", device])
            .output()
            .expect("failed to execute process");

        println!("{:?} {:?} {:?}", start_mon.status, output, check_mode);
    }

    // Change channel to 6
    let _ = Command::new("iwconfig")
        .args([device, "channel", "6"])
        .output()
        .expect("failed to execute process");

    Ok(())
}

pub fn disable_monitor_mode(device: &str) -> Result<(), String> {
    // Disable monitoring mode using airmon-ng
    let stop_mon = Command::new("sudo")
        .args(["airmon-ng", "stop", device])
        .output()
        .expect("failed to execute process");

    Ok(())
}
