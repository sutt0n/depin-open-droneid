extern crate packed_struct;
use nom::bytes::complete::take;
use nom::number::complete::{le_u16, le_u8};
use nom::IResult;
use radiotap::Radiotap;
use std::convert::TryInto;

pub const WIFI_ALLIANCE_OUI: [u8; 3] = [0x50, 0x6f, 0x9a];
pub const NAN_SERVICE_ID: [u8; 6] = [0x88, 0x69, 0x19, 0x9d, 0x92, 0x09];

#[derive(Debug)]
pub struct ActionFrame<'a> {
    pub frame_control: u16,
    pub frame_control_version: u8, // first 2 bits, 000000xx
    pub frame_control_type: u8, // next 2 bits, 0000xx00
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
pub struct ServiceDescriptorAttribute<'a> {
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
pub struct OpenDroneIDMessagePack {
    // 4 bits
    pub message_type: u8,
    // 4 bits
    pub version: u8,
    pub single_msg_size: u8,
    pub num_messages: u8,
    pub messages: Vec<OpenDroneIDMessage>,
}

#[derive(Debug)]
pub struct OpenDroneIDMessage {
    pub message_type: u8,
    pub version: u8,
    pub message_body: [u8; 25],
}

pub fn parse_open_drone_id_message_pack(input: &[u8]) -> IResult<&[u8], OpenDroneIDMessagePack> {
    let (input, message_type_and_version) = le_u8(input)?;
    let message_type = message_type_and_version >> 4;
    let version = message_type_and_version & 0x0F;

    let (input, single_msg_size) = le_u8(input)?;
    let (input, num_messages) = le_u8(input)?;

    let mut messages = Vec::new();

    for _ in 0..num_messages {
        let (input, message_type_and_version) = le_u8(input)?;
        let message_type = message_type_and_version >> 4;
        let version = message_type_and_version & 0x0F;

        let (_, message_body) = take(25usize)(input)?;

        messages.push(OpenDroneIDMessage {
            message_type,
            version,
            message_body: message_body.try_into().unwrap(),
        });
    }

    Ok((
        input,
        OpenDroneIDMessagePack {
            message_type,
            version,
            single_msg_size,
            num_messages,
            messages,
        },
    ))
}

pub fn parse_service_descriptor_attribute(input: &[u8]) -> IResult<&[u8], ServiceDescriptorAttribute> {
    let (input, attribute_id) = le_u8(input)?;
    let (input, attribute_length) = le_u16(input)?;
    let (input, service_id) = take(6usize)(input)?;
    let (input, instance_id) = le_u8(input)?;
    let (input, requestor_id) = le_u8(input)?;
    let (input, service_control) = le_u8(input)?;
    let (input, service_info_length) = le_u8(input)?;
    let (input, message_counter) = le_u8(input)?;
    let (input, service_info) = take(service_info_length - 1)(input)?;

    Ok((
        input,
        ServiceDescriptorAttribute {
            attribute_id,
            attribute_length,
            service_id: service_id.try_into().unwrap(),
            instance_id,
            requestor_id,
            service_control,
            service_info_length,
            message_counter,
            service_info,
        },
    ))
}

pub fn parse_action_frame(input: &[u8]) -> IResult<&[u8], ActionFrame> {
    let (input, frame_control) = le_u16(input)?;
    let frame_control_version = (frame_control & 0b00000011) as u8;
    let frame_control_type = ((frame_control & 0b00001100) >> 2) as u8;
    let frame_control_subtype = ((frame_control & 0b11110000) >> 4) as u8;
    let (input, duration_id) = le_u16(input)?;
    let (input, address1) = take(6usize)(input)?;
    let (input, address2) = take(6usize)(input)?;
    let (input, address3) = take(6usize)(input)?;
    let (input, sequence_control) = le_u16(input)?;
    let (input, category) = take(1usize)(input)?;
    let (input, action) = take(1usize)(input)?;
    let (input, oui) = take(3usize)(input)?;
    let (input, oui_type) = take(1usize)(input)?;
    let (input, body) = take(input.len())(input)?;
    
    Ok((
        input,
        ActionFrame {
            frame_control,
            frame_control_version,
            frame_control_type,
            frame_control_subtype,
            duration_id,
            address1,
            address2,
            address3,
            sequence_control,
            category: category[0],
            action: action[0],
            oui: oui.try_into().unwrap(),
            oui_type: oui_type[0],
            body,
        },
    ))
}

pub fn remove_radiotap_header(input: &[u8]) -> &[u8] {
    let radiotap: Option<Radiotap> = match Radiotap::from_bytes(input) {
        Ok(radiotap) => Some(radiotap),
        Err(error) => {
            println!(
                "Couldn't read packet data with Radiotap: {:?}, error {error:?}",
                &input
            );
            None
        }
    };

    if let Some(radiotap) = radiotap {
        &input[radiotap.header.length..]
    } else {
        input
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{self, Read, BufReader};
    use radiotap::Radiotap;

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
    fn test_parse_action_frame() {
        // file content is an array of bytes in [1,2,3,4] format
        let wifi_data: Vec<u8> = read_fixture("fixtures/wifi_packet_data.txt").unwrap();

        // vec<u8> to slice
        let frame_data = &wifi_data[..];

        let payload = remove_radiotap_header(frame_data);

        let action_frame = match parse_action_frame(payload) {
            Ok((_, frame)) => frame,
            Err(e) => {
                eprintln!("Failed to parse IEEE 802.11 action frame: {:?}", e);
                return;
            }
        };

        assert_eq!(action_frame.frame_control, 0xd0);
        assert_eq!(action_frame.frame_control_version, 0x0);
        assert_eq!(action_frame.frame_control_type, 0x0);
        assert_eq!(action_frame.frame_control_subtype, 0xd);
        assert_eq!(action_frame.oui, WIFI_ALLIANCE_OUI);
        assert_eq!(action_frame.oui_type, 0x13);

        // let open_drone_id_message_pack: Option<OpenDroneIDMessagePack> = match parse_action_frame(payload) {
        //     Ok((_, frame)) => {
        //         match parse_service_descriptor_attribute(frame.body) {
        //             Ok((_, attribute)) => {
        //                 match parse_open_drone_id_message_pack(attribute.service_info) {
        //                     Ok((_, message_pack)) => {
        //                         Some(message_pack)
        //                     }
        //                     Err(e) => {
        //                         eprintln!("Failed to parse Open Drone ID message pack: {:?}", e);
        //                         None
        //                     },
        //                 }
        //             }
        //             Err(e) => {
        //                 eprintln!("Failed to parse service descriptor attribute: {:?}", e);
        //                 None
        //             }
        //         }
        //     }
        //     Err(e) => {
        //         eprintln!("Failed to parse IEEE 802.11 action frame: {:?}", e);
        //         None
        //     }
        // };
        //
        // assert_eq!(open_drone_id_message_pack.is_some(), true);
        //
        // let open_drone_id_message_pack = open_drone_id_message_pack.unwrap();
        //
        // assert_eq!(open_drone_id_message_pack.message_type, 0xf);
        // assert_eq!(open_drone_id_message_pack.version, 0x2);
    }

    #[test]
    fn test_parse_service_descriptor_attribute() {
        let wifi_data: Vec<u8> = read_fixture("fixtures/wifi_packet_data.txt").unwrap();

        // vec<u8> to slice
        let frame_data = &wifi_data[..];

        let payload = remove_radiotap_header(frame_data);

        let service_descriptor_attribute: Option<ServiceDescriptorAttribute> = match parse_action_frame(payload) {
            Ok((_, frame)) => {
                match parse_service_descriptor_attribute(frame.body) {
                    Ok((_, attribute)) => {
                        Some(attribute)
                    }
                    Err(e) => {
                        eprintln!("Failed to parse service descriptor attribute: {:?}", e);
                        None
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to parse IEEE 802.11 action frame: {:?}", e);
                None
            }
        };

        assert_eq!(service_descriptor_attribute.is_some(), true);

        let service_descriptor_attribute = service_descriptor_attribute.unwrap();

        assert_eq!(service_descriptor_attribute.attribute_id, 0x3);
        assert_eq!(service_descriptor_attribute.service_id, NAN_SERVICE_ID);
        assert_eq!(service_descriptor_attribute.instance_id, 0x1);
        assert_eq!(service_descriptor_attribute.requestor_id, 0x0);
        assert_eq!(service_descriptor_attribute.service_control, 0x10);
    }

    #[test]
    fn test_parse_open_drone_id_message_pack() {
        let wifi_data: Vec<u8> = read_fixture("fixtures/wifi_packet_data.txt").unwrap();

        // vec<u8> to slice
        let frame_data = &wifi_data[..];

        let payload = remove_radiotap_header(frame_data);

        let open_drone_id_message_pack: Option<OpenDroneIDMessagePack> = match parse_action_frame(payload) {
            Ok((_, frame)) => {
                match parse_service_descriptor_attribute(frame.body) {
                    Ok((_, attribute)) => {
                        match parse_open_drone_id_message_pack(attribute.service_info) {
                            Ok((_, message_pack)) => {
                                Some(message_pack)
                            }
                            Err(e) => {
                                eprintln!("Failed to parse Open Drone ID message pack: {:?}", e);
                                None
                            },
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse service descriptor attribute: {:?}", e);
                        None
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to parse IEEE 802.11 action frame: {:?}", e);
                None
            }
        };

        assert_eq!(open_drone_id_message_pack.is_some(), true);

        let open_drone_id_message_pack = open_drone_id_message_pack.unwrap();

        assert_eq!(open_drone_id_message_pack.message_type, 0xf);
        assert!(open_drone_id_message_pack.version <= 0xf); // 0x0 <= x <= 0xf
        assert_eq!(open_drone_id_message_pack.single_msg_size, 0x19);
        assert_eq!(open_drone_id_message_pack.num_messages, 0x4);
    }
}
