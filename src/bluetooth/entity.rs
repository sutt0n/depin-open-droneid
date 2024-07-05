#[derive(Debug)]
pub struct BluetoothAdvertisementFrame {
    pub app_code: u8,
    pub counter: u8,
    pub message: [u8; 25],
}
