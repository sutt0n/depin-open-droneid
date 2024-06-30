use packed_struct::prelude::*;
use serde::{Deserialize, Deserializer};
use serde_bytes::ByteBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct NanServiceDiscoveryFrame {
    pub category: u8,
    pub action: u8,
    pub oui: [u8; 3],
    pub oui_type: u8,
    // pub nan_attributes: ByteBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NanServiceDescriptorAttribute {
    pub attribute_id: u8,
    pub length: u16,
    pub service_id: [u8; 6],
    pub instance_id: u8,
    pub requestor_instance_id: u8,
    pub service_control: u8,
    pub service_info_length: u8,
    pub message_counter: u8,
    pub service_info: ByteBuf, 
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpenDroneIDMessagePack {
    pub msg_type: u8,
    pub version: u8,
    pub single_msg_size: u8,
    pub num_of_msgs_in_pack: u8,
    pub messages: ByteBuf,
}

#[derive(PackedStruct)]
#[packed_struct(bit_numbering = "msb0")]
pub struct OpenDroneIdMessage {
    #[packed_field(bits = "0..=3", ty = "enum")]
    pub msg_type: DroneMessageType,
    #[packed_field(bits = "4..=7")]
    pub version: u8,
    pub message: [u8; 24],
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum DroneMessageType {
    BasicID = 0,
    Location = 1,
    Auth = 2,
    SelfID = 3,
    System = 4,
    OperatorID = 5,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{self, Read, BufReader};
    use libwifi::Addresses;
    use pcap::Packet;
    use radiotap::Radiotap;
    use libwifi::frame::ProbeResponse;
    use libwifi::frame::Frame;

    fn read_fixture(file_path: &str) -> io::Result<Vec<u8>> {
        // Open the file
        let file = File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);
    
        // Read the file content into a string
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;
    
        // Trim the square brackets and split the string by comma
        let content = content.trim().trim_start_matches('[').trim_end_matches(']');
        let bytes: Vec<u8> = content
            .split(',')
            .map(|s| s.trim().parse().expect("Failed to parse byte"))
            .collect();
    
        Ok(bytes)
    }

    #[test]
    fn test_parse_nan_service_discovery_frame() {
        // file content is an array of bytes in [1,2,3,4] format
        let wifi_data: Vec<u8> = read_fixture("fixtures/wifi_packet_data.txt").unwrap();

        // vec<u8> to slice
        let wifi_data = &wifi_data[..];

        let radiotap = match Radiotap::from_bytes(wifi_data) {
            Ok(radiotap) => radiotap,
            Err(error) => {
                println!(
                    "Couldn't read packet data with Radiotap: {:?}, error {error:?}",
                    &wifi_data
                );
                return ();
            }
        };

        println!("Radiotap header {:?}", radiotap.header.length);

        let payload = &wifi_data[radiotap.header.length..];

        // println!("wifi_data: {:?}", frame.header);

        // let nan_service_discovery_frame: NanServiceDiscoveryFrame = bincode::deserialize(wifi_data).unwrap();

        // assert_eq!(nan_service_discovery_frame.category, 0x04);
        // assert_eq!(nan_service_discovery_frame.action, 0);
        // assert_eq!(nan_service_discovery_frame.oui, [0, 0, 0]);
        // assert_eq!(nan_service_discovery_frame.oui_type, 0);
        // assert_eq!(nan_service_discovery_frame.nan_attributes.len(), 0);
    }
}
