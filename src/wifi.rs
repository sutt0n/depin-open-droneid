use std::process::Command as SysCommand;

pub fn enable_monitor_mode(device: &str) -> Result<(), String> {
    // Check if the device is already in monitoring mode
    let check_mode = SysCommand::new("iwconfig")
        .arg(device)
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8_lossy(&check_mode.stdout);

    if !output.contains("Mode:Monitor") {
        // Enable monitoring mode using airmon-ng
        let start_mon = SysCommand::new("sudo")
            .args(["airmon-ng", "start", device])
            .output()
            .expect("failed to execute process");

        println!("{:?} {:?}", start_mon.status, output);

        if start_mon.status.success() {
            println!("Monitoring mode enabled on {}", device);
        } else {
            return Err("Failed to enable monitoring mode".to_string());
        }
    }

    Ok(())
}

pub fn disable_monitor_mode(device: &str) -> Result<(), String> {
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
