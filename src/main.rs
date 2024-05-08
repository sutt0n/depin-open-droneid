use pcap::{Capture, Device};
use clap::{Command, Arg};

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

    println!("Using device: {}", device_name);

    let mut cap = Capture::from_device(device_name.as_str()).unwrap()
        .promisc(true)
        .snaplen(5000)
        .open();

    if let Err(e) = cap {
        eprintln!("error opening device \"{}\": {}", device_name, e);
        let devices = Device::list().unwrap();
        let device_names = devices.iter().map(|d| d.name.clone()).collect::<Vec<String>>();

        eprintln!("available devices: {:?}", device_names);
        return;
    }

    while let Ok(packet) = cap.as_mut().unwrap().next_packet() {
        println!("received packet: {:?}", packet);
    }
}
