use std::{process::Command, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use log::{debug, trace};
use tokio::{sync::Mutex, time::sleep};

use super::WifiConfig;

#[derive(Debug, Clone)]
pub struct WifiInterface {
    pub name: String,
    pub channel: u64,
    pub last_odid_received: Option<DateTime<Utc>>,
    pub channel_mod_freq_ms: u64,
}

impl Default for WifiInterface {
    fn default() -> Self {
        WifiInterface {
            name: "wlan0".to_string(),
            channel: 6,
            last_odid_received: None,
            channel_mod_freq_ms: 1000,
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
            channel_mod_freq_ms: config.channel_mod_freq_ms,
        };

        // Enable monitoring mode
        enable_monitor_mode(wifi_interface.name.as_str()).unwrap();

        Ok(wifi_interface)
    }

    pub async fn run_loop(interface: Arc<Mutex<WifiInterface>>) -> anyhow::Result<()> {
        loop {
            let mut wifi_interface = interface.lock().await;
            if wifi_interface.should_change_channel() {
                wifi_interface.adjust_channel();
            }

            sleep(Duration::from_millis(wifi_interface.channel_mod_freq_ms)).await;
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
            None => {
                let time_diff = current_time.signed_duration_since(
                    Utc::now() - chrono::Duration::seconds(WifiInterface::TIME_TO_CHANGE_CHANNEL),
                );

                time_diff.num_seconds() > 5
            }
        }
    }

    pub fn adjust_channel(&mut self) {
        match self.channel {
            // 2.4 GHz channels
            1 => self.channel = 6,
            6 => self.channel = 11,
            11 => self.channel = 1,
            // 5 GHz channels
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
