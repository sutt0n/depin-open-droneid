extern crate packed_struct;
use packed_struct::prelude::*;
use std::convert::TryInto;

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb")]
pub struct IEEE80211Header {
    #[packed_field(bits = "0..=1")]
    protocol_version: u8,
    #[packed_field(bits = "2..=3")]
    type_field: u8,
    #[packed_field(bits = "4..=7")]
    subtype: u8,
    #[packed_field(bits = "8")]
    to_ds: bool,
    #[packed_field(bits = "9")]
    from_ds: bool,
    #[packed_field(bits = "10")]
    more_frag: bool,
    #[packed_field(bits = "11")]
    retry: bool,
    #[packed_field(bits = "12")]
    power_mgmt: bool,
    #[packed_field(bits = "13")]
    more_data: bool,
    #[packed_field(bits = "14")]
    protected_frame: bool,
    #[packed_field(bits = "15")]
    order: bool,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb")]
pub struct ManagementFrameBody {
    #[packed_field(bits = "0..=15")]
    duration: u16,
    #[packed_field(bits = "16..=63")]
    receiver_address: [u8; 6],
    #[packed_field(bits = "64..=111")]
    transmitter_address: [u8; 6],
    #[packed_field(bits = "112..=159")]
    bssid: [u8; 6],
    #[packed_field(bits = "160..=175")]
    sequence_control: u16,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb")]
pub struct ActionFrameHeader {
    #[packed_field(bits = "0..=7")]
    category: u8,
    #[packed_field(bits = "8..=15")]
    action_code: u8,
    #[packed_field(bits = "16..=23")]
    dialog_token: u8,
    #[packed_field(bits = "24..=31")]
    nan_type: u8,
    #[packed_field(bits = "32..=47")]
    nan_length: u16,
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
        let frame_data = &wifi_data[..];

        let radiotap: Option<Radiotap> = match Radiotap::from_bytes(frame_data) {
        Ok(radiotap) => Some(radiotap),
        Err(error) => {
            println!(
                "Couldn't read packet data with Radiotap: {:?}, error {error:?}",
                &frame_data
            );
            None
        }
    };

        let payload = &frame_data[radiotap.unwrap().header.length..];

        println!("Radiotap header {:?}", payload);

        // let header = IEEE80211Header::unpack(&payload[0..2].try_into().unwrap()).unwrap();
        // let body = ManagementFrameBody::unpack(&frame_data[2..20].try_into().unwrap()).unwrap();
        // let action_header = ActionFrameHeader::unpack(&frame_data[20..28].try_into().unwrap()).unwrap();

        // let nan_length = action_header.nan_length as usize;
        // let nan_data = &frame_data[28..28 + nan_length];

        // println!("wifi_data: {:?}", header);

        // let nan_service_discovery_frame: NanServiceDiscoveryFrame = bincode::deserialize(wifi_data).unwrap();

        // assert_eq!(nan_service_discovery_frame.category, 0x04);
        // assert_eq!(nan_service_discovery_frame.action, 0);
        // assert_eq!(nan_service_discovery_frame.oui, [0, 0, 0]);
        // assert_eq!(nan_service_discovery_frame.oui_type, 0);
        // assert_eq!(nan_service_discovery_frame.nan_attributes.len(), 0);
    }
}
