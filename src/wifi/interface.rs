use std::process::Command;

use chrono::{DateTime, Utc};
use log::{debug, trace};

use super::WifiConfig;

#[derive(Debug, Clone)]
pub struct WifiInterface {
    pub name: String,
    pub channel: u64,
    pub last_odid_received: Option<DateTime<Utc>>,
}

impl Default for WifiInterface {
    fn default() -> Self {
        WifiInterface {
            name: "wlan0".to_string(),
            channel: 6,
            last_odid_received: None,
        }
    }
}

impl WifiInterface {
    // Time in seconds to change the channel
    const TIME_TO_CHANGE_CHANNEL: i64 = 30;

    pub async fn init(config: WifiConfig) -> anyhow::Result<Self> {
        let wifi_interface = WifiInterface {
            name: config.device_name,
            channel: config.channels[0],
            last_odid_received: None,
        };

        // Enable monitoring mode
        enable_monitor_mode(wifi_interface.name.as_str()).unwrap();

        Ok(wifi_interface)
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            // Check if the channel should be changed
            if self.should_change_channel() {
                self.adjust_channel();
            }
        }
    }

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
            // 2.4 GHz channels
            1 => self.channel = 6,
            6 => self.channel = 11,
            11 => self.channel = 36,
            // 5 GHz channels
            36 => self.channel = 40,
            40 => self.channel = 44,
            44 => self.channel = 48,
            48 => self.channel = 149,
            149 => self.channel = 153,
            153 => self.channel = 157,
            157 => self.channel = 161,
            161 => self.channel = 1,
            _ => self.channel = 6,
        }

        debug!("Adjusting channel to {}", self.channel);

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
