pub const WIFI_ALLIANCE_OUI: [u8; 3] = [0x50, 0x6f, 0x9a];
pub const ASDSTAN_OUI: [u8; 3] = [0xfa, 0x0b, 0xbc];
pub const NAN_SERVICE_ID: [u8; 6] = [0x88, 0x69, 0x19, 0x9d, 0x92, 0x09];

#[derive(Debug)]
pub struct WifiActionFrame<'a> {
    pub frame_control: u16,
    pub frame_control_version: u8, // first 2 bits, 000000xx
    pub frame_control_type: u8,    // next 2 bits, 0000xx00
    pub frame_control_subtype: u8, // next 4 bits, xxxx0000
    pub duration_id: u16,
    pub address1: &'a [u8],
    pub address2: &'a [u8],
    pub address3: &'a [u8],
    pub sequence_control: u16,
    pub category: u8,
    pub action: u8,
    pub oui: [u8; 3], // wi-fi alliance, 0x50, 0x6f, 0x9a
    pub oui_type: u8,
    pub body: &'a [u8],
}

#[derive(Debug)]
pub struct WifiBeaconFrame<'a> {
    pub frame_control: u16,
    pub duration: u16,
    pub destination_address: [u8; 6],
    pub source_address: [u8; 6],
    pub bssid: [u8; 6],
    pub sequence_control: u16,
    pub vendor_specific_data: &'a [u8],
}

#[derive(Debug)]
pub struct WifiServiceDescriptorAttribute<'a> {
    pub attribute_id: u8,
    pub attribute_length: u16,
    pub service_id: &'a [u8],
    pub instance_id: u8,
    pub requestor_id: u8,
    pub service_control: u8,
    pub service_info_length: u8,
    pub service_info: &'a [u8],
    pub message_counter: u8,
}

#[derive(Debug)]
pub struct WifiOpenDroneIDMessagePack {
    pub message_type: u8, // 4 bits
    pub version: u8,      // 4 bits
    pub single_msg_size: u8,
    pub num_messages: u8,
    pub messages: Vec<WifiOpenDroneIDMessage>,
}

#[derive(Debug)]
pub struct WifiOpenDroneIDMessage {
    pub message_type: u8,
    pub version: u8,
    pub message_body: [u8; 24],
}

// todo: move to repo.rs
//#[cfg(test)]
//pub mod tests {
//    use crate::odid::{parse_location, Location};
//    use crate::wifi::{
//        parse_action_frame, parse_beacon_frame, parse_open_drone_id_message_pack,
//        parse_service_descriptor_attribute, remove_radiotap_header,
//    };
//
//    use super::*;
//    use std::fs::File;
//    use std::io::{self, BufReader, Read};
//
//    fn read_fixture(file_path: &str) -> io::Result<Vec<u8>> {
//        // Open the file
//        let file = File::open(file_path)?;
//        let mut buf_reader = BufReader::new(file);
//
//        // Read the file content into a string
//        let mut content = String::new();
//        buf_reader.read_to_string(&mut content)?;
//
//        // Trim the square brackets and split the string by comma
//        let content = content.trim().trim_start_matches('[').trim_end_matches(']');
//        let bytes: Vec<u8> = content
//            .split(',')
//            .map(|s| s.trim().parse().expect("Failed to parse byte"))
//            .collect();
//
//        Ok(bytes)
//    }
//
//    #[test]
//    fn test_parse_beacon_frame() {
//        let wifi_data: Vec<u8> = read_fixture("fixtures/wlan_beacon_packet_data.txt").unwrap();
//
//        let frame_data = &wifi_data[..];
//
//        let payload = remove_radiotap_header(frame_data);
//
//        let beacon_frame: Option<WifiBeaconFrame> = match parse_beacon_frame(payload) {
//            Ok((_, frame)) => Some(frame),
//            Err(e) => {
//                eprintln!("Failed to parse IEEE 802.11 beacon frame: {:?}", e);
//                None
//            }
//        };
//
//        assert_eq!(beacon_frame.is_some(), true);
//
//        let beacon_frame = beacon_frame.unwrap();
//
//        let odid_message_pack: Option<WifiOpenDroneIDMessagePack> =
//            match parse_open_drone_id_message_pack(beacon_frame.vendor_specific_data) {
//                Ok((_, message_pack)) => Some(message_pack),
//                Err(e) => {
//                    eprintln!("Failed to parse Open Drone ID message pack: {:?}", e);
//                    None
//                }
//            };
//
//        assert_eq!(odid_message_pack.is_some(), true);
//
//        let open_drone_id_message_pack = odid_message_pack.unwrap();
//
//        assert_eq!(open_drone_id_message_pack.version, 0x02);
//        assert_eq!(open_drone_id_message_pack.message_type, 0xf);
//        assert!(open_drone_id_message_pack.version <= 0xf); // 0x0 <= x <= 0xf
//        assert_eq!(open_drone_id_message_pack.single_msg_size, 0x19);
//        assert_eq!(open_drone_id_message_pack.num_messages, 0x4);
//
//        // first message basic id
//        assert_eq!(open_drone_id_message_pack.messages[0].message_type, 0x0);
//
//        // second message system message
//        assert_eq!(open_drone_id_message_pack.messages[1].message_type, 0x4);
//
//        // third message location
//        assert_eq!(open_drone_id_message_pack.messages[2].message_type, 0x1);
//
//        let location: Option<Location> =
//            match parse_location(&open_drone_id_message_pack.messages[2].message_body) {
//                Ok((_, location)) => Some(location),
//                Err(e) => {
//                    eprintln!("Failed to parse location message: {:?}", e);
//                    None
//                }
//            };
//
//        assert_eq!(location.is_some(), true);
//
//        let location = location.unwrap();
//
//        assert_eq!(location.latitude_int, 358025796);
//        assert_eq!(location.longitude_int, -907110086);
//
//        // fourth message self id
//        assert_eq!(open_drone_id_message_pack.messages[3].message_type, 0x3);
//    }
//
//    #[test]
//    fn test_parse_action_frame() {
//        // file content is an array of bytes in [1,2,3,4] format
//        let wifi_data: Vec<u8> = read_fixture("fixtures/wifi_packet_data.txt").unwrap();
//
//        // vec<u8> to slice
//        let frame_data = &wifi_data[..];
//
//        let payload = remove_radiotap_header(frame_data);
//
//        let action_frame: Option<WifiActionFrame> = match parse_action_frame(payload) {
//            Ok((_, frame)) => Some(frame),
//            Err(e) => {
//                eprintln!("Failed to parse IEEE 802.11 action frame: {:?}", e);
//                None
//            }
//        };
//
//        assert_eq!(action_frame.is_some(), true);
//
//        let action_frame = action_frame.unwrap();
//
//        assert_eq!(action_frame.frame_control, 0xd0);
//        assert_eq!(action_frame.frame_control_version, 0x0);
//        assert_eq!(action_frame.frame_control_type, 0x0);
//        assert_eq!(action_frame.frame_control_subtype, 0xd);
//        assert_eq!(action_frame.oui, WIFI_ALLIANCE_OUI);
//        assert_eq!(action_frame.oui_type, 0x13);
//    }
//
//    #[test]
//    fn test_parse_service_descriptor_attribute() {
//        let wifi_data: Vec<u8> = read_fixture("fixtures/wifi_packet_data.txt").unwrap();
//
//        // vec<u8> to slice
//        let frame_data = &wifi_data[..];
//
//        let payload = remove_radiotap_header(frame_data);
//
//        let service_descriptor_attribute: Option<WifiServiceDescriptorAttribute> =
//            match parse_action_frame(payload) {
//                Ok((_, frame)) => match parse_service_descriptor_attribute(frame.body) {
//                    Ok((_, attribute)) => Some(attribute),
//                    Err(e) => {
//                        eprintln!("Failed to parse service descriptor attribute: {:?}", e);
//                        None
//                    }
//                },
//                Err(e) => {
//                    eprintln!("Failed to parse IEEE 802.11 action frame: {:?}", e);
//                    None
//                }
//            };
//
//        assert_eq!(service_descriptor_attribute.is_some(), true);
//
//        let service_descriptor_attribute = service_descriptor_attribute.unwrap();
//
//        assert_eq!(service_descriptor_attribute.attribute_id, 0x3);
//        assert_eq!(service_descriptor_attribute.service_id, NAN_SERVICE_ID);
//        assert_eq!(service_descriptor_attribute.instance_id, 0x1);
//        assert_eq!(service_descriptor_attribute.requestor_id, 0x0);
//        assert_eq!(service_descriptor_attribute.service_control, 0x10);
//    }
//
//    #[test]
//    fn test_parse_open_drone_id_message_pack() {
//        let wifi_data: Vec<u8> = read_fixture("fixtures/wifi_packet_data.txt").unwrap();
//
//        // vec<u8> to slice
//        let frame_data = &wifi_data[..];
//
//        let payload = remove_radiotap_header(frame_data);
//
//        let open_drone_id_message_pack: Option<WifiOpenDroneIDMessagePack> =
//            match parse_action_frame(payload) {
//                Ok((_, frame)) => match parse_service_descriptor_attribute(frame.body) {
//                    Ok((_, attribute)) => {
//                        match parse_open_drone_id_message_pack(attribute.service_info) {
//                            Ok((_, message_pack)) => Some(message_pack),
//                            Err(e) => {
//                                eprintln!("Failed to parse Open Drone ID message pack: {:?}", e);
//                                None
//                            }
//                        }
//                    }
//                    Err(e) => {
//                        eprintln!("Failed to parse service descriptor attribute: {:?}", e);
//                        None
//                    }
//                },
//                Err(e) => {
//                    eprintln!("Failed to parse IEEE 802.11 action frame: {:?}", e);
//                    None
//                }
//            };
//
//        assert_eq!(open_drone_id_message_pack.is_some(), true);
//
//        let open_drone_id_message_pack = open_drone_id_message_pack.unwrap();
//
//        assert_eq!(open_drone_id_message_pack.message_type, 0xf);
//        assert!(open_drone_id_message_pack.version <= 0xf); // 0x0 <= x <= 0xf
//        assert_eq!(open_drone_id_message_pack.single_msg_size, 0x19);
//        assert_eq!(open_drone_id_message_pack.num_messages, 0x4);
//
//        // first message basic id
//        assert_eq!(open_drone_id_message_pack.messages[0].message_type, 0x0);
//
//        // second message system message
//        assert_eq!(open_drone_id_message_pack.messages[1].message_type, 0x4);
//
//        // third message location
//        assert_eq!(open_drone_id_message_pack.messages[2].message_type, 0x1);
//
//        let location: Option<Location> =
//            match parse_location(&open_drone_id_message_pack.messages[2].message_body) {
//                Ok((_, location)) => Some(location),
//                Err(e) => {
//                    eprintln!("Failed to parse location message: {:?}", e);
//                    None
//                }
//            };
//
//        assert_eq!(location.is_some(), true);
//
//        let location = location.unwrap();
//
//        assert_eq!(location.latitude_int, 358026271);
//        assert_eq!(location.longitude_int, -907113683);
//
//        // fourth message self id
//        assert_eq!(open_drone_id_message_pack.messages[3].message_type, 0x3);
//    }
//}
