use pcap::{Capture, Device};
use std::process::Command as SysCommand;
use clap::{Command, Arg};

mod messages;
mod parsers;

use crate::messages::RemoteIdMessage;
use crate::parsers::{parse_basic_id, parse_location, parse_authentication};

fn enable_monitor_mode(device: &str) -> Result<(), String> {
    // Check if the device is already in monitoring mode
    let check_mode = SysCommand::new("iwconfig")
        .arg(device)
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8_lossy(&check_mode.stdout);

    if !output.contains("Monitor mode enabled") {
        // Enable monitoring mode using airmon-ng
        let start_mon = SysCommand::new("sudo")
            .args(["airmon-ng", "start", device])
            .output()
            .expect("failed to execute process");

        if start_mon.status.success() {
            println!("Monitoring mode enabled on {}", device);
        } else {
            return Err("Failed to enable monitoring mode".to_string());
        }
    }

    Ok(())
}

fn disable_monitor_mode(device: &str) -> Result<(), String> {
    // Disable monitoring mode using airmon-ng
    let stop_mon = SysCommand::new("sudo")
        .args(["airmon-ng", "stop", device])
        .output()
        .expect("failed to execute process");

    if stop_mon.status.success() {
        println!("Monitoring mode disabled on {}", device);
    } else {
        return Err("Failed to disable monitoring mode".to_string());
    }

    Ok(())
}

fn main() {
    let matches = Command::new("Drone ID Scanner")
        .version("1.0")
        .author("Your Name")
        .about("Scans for Open Drone ID packets")
        .arg(Arg::new("device")
             .short('d')
             .long("device")
             .default_value("wlan0")
             .help("The network device to capture packets from (e.g., wlan0)"))
        .get_matches();

    let device_name = matches.get_one::<String>("device").unwrap();

    // if let Err(e) = enable_monitor_mode(device_name) {
    //     eprintln!("Error: {}", e);
    //     return;
    // }

    println!("Using device: {}", device_name);

    let mut cap = Capture::from_device(device_name.as_str()).unwrap()
        .rfmon(true)
        .open();

    if let Err(e) = cap {
        eprintln!("error opening device \"{}\": {}", device_name, e);
        let devices = Device::list().unwrap();
        let device_names = devices.iter().map(|d| d.name.clone()).collect::<Vec<String>>();

        eprintln!("available devices: {:?}", device_names);
        return;
    }

    while let Ok(packet) = cap.as_mut().unwrap().next_packet() {
        let data = packet.data;

        println!("Received packet with {} bytes", data.len());
        println!("Data: {:?}", data);

        // packet bytes are in little-endian order
        let data = data.iter().enumerate().map(|(i, &b)| {
            if i % 2 == 0 {
                b
            } else {
                b.rotate_left(4)
            }
        }).collect::<Vec<u8>>();
        
        let data = &data[..];

        // header is 1 byte
        // bits 7..4 are the message type 
        // bytes 3..0 are the protocol version
        let message_type = data[0] >> 4;
        let protocol_version = data[0] & 0x0F;

        println!("Received packet with protocol version {} and message type {}", protocol_version, message_type);

        let message = match message_type {
            0 => RemoteIdMessage::BasicId(parse_basic_id(data)),
            1 => RemoteIdMessage::Location(parse_location(data)),
            2 => RemoteIdMessage::Authentication(parse_authentication(data)),
            _ => RemoteIdMessage::Unknown,
        };

        println!("Received packet: {:?}", message);
    }

    println!("Exiting...");

    disable_monitor_mode(device_name).unwrap_or_else(|err| eprintln!("Error: {}", err));
}
