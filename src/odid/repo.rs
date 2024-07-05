use byteorder::{ByteOrder, LittleEndian};
use nom::bytes::complete::{take, take_while};
use nom::number::complete::{be_i32, le_i16, le_i32, le_u16, le_u8};
use nom::number::streaming::le_f32;
use nom::IResult;

use super::{
    BasicId, Location, Operator, OperatorLocationType, RemoteIdMessage, SystemMessage, UaType,
    UasIdType,
};

pub fn parse_message_type(input: &[u8]) -> IResult<&[u8], RemoteIdMessage> {
    match take(4usize)(input) {
        Ok((_, message_type)) => Ok((input, RemoteIdMessage::from(message_type[0] >> 4))),
        Err(e) => Err(e),
    }
}

pub fn parse_basic_id(input: &[u8]) -> IResult<&[u8], BasicId> {
    let (input, id_and_ua_type) = le_u8(input)?;
    let id_type = (id_and_ua_type & 0xF0) >> 4;
    let ua_type = id_and_ua_type & 0x0F;

    // iterate over the bytes until we find a null byte
    let (input, uas_id) = take_while(|b: u8| b != 0x0)(input)?;

    // filter out non-alphanumeric characters
    let uas_id = uas_id
        .iter()
        .cloned()
        .collect::<Vec<u8>>()
        .iter()
        .filter(|b| b.is_ascii_alphanumeric())
        .cloned()
        .collect::<Vec<u8>>();

    Ok((
        input,
        BasicId {
            uas_id_type: UasIdType::from(id_type),
            ua_type: UaType::from(ua_type),
            uas_id: String::from_utf8_lossy(&uas_id).to_string(),
        },
    ))
}

pub fn parse_system_message(input: &[u8]) -> IResult<&[u8], SystemMessage> {
    let (input, flags) = le_u8(input)?;
    let operator_location_type = flags & 0x03;

    let operator_location_type = OperatorLocationType::from(operator_location_type);

    // operator_latitude_int is 4 bytes, signed int, little endian
    let (input, operator_latitude_int) = le_i32(input)?;
    // let (input, operator_latitude_int) = take(32usize)(input)?;
    let (input, operator_longitude_int) = le_i32(input)?;
    let (input, area_count) = le_i16(input)?;
    let (input, area_radius) = le_u8(input)?;
    let (input, area_ceiling) = le_u16(input)?;
    let (input, area_floor) = le_u16(input)?;

    Ok((
        input,
        SystemMessage {
            operator_location_type,
            operator_latitude_int,
            operator_longitude_int,
            area_count,
            area_radius,
            area_ceiling,
            area_floor,
        },
    ))
}

pub fn parse_operator_id(input: &[u8]) -> IResult<&[u8], Operator> {
    let (input, operator_id_type) = le_u8(input)?;
    let (input, operator_id) = take_while(|b| b != 0x00)(input)?;

    Ok((
        input,
        Operator {
            operator_id_type,
            operator_id: String::from_utf8_lossy(operator_id).to_string(),
        },
    ))
}

pub fn parse_location(input: &[u8]) -> IResult<&[u8], Location> {
    let (input, input_first_byte) = le_u8(input)?;

    let status = (input_first_byte & 0xF0) >> 4;
    let _reserved = input_first_byte & 0x08;
    let height_type = input_first_byte & 0x04;
    let ew_direction = input_first_byte & 0x02;
    let speed_multiplier = input_first_byte & 0x01;

    let (input, tracking_direction) = le_u8(input)?;
    let (input, speed) = le_u8(input)?;
    let (input, vertical_speed) = le_u8(input)?;
    let (input, latitude_int) = le_i32(input)?;
    let (input, longitude_int) = le_i32(input)?;
    let (input, altitude_pressure) = le_u16(input)?;
    let (input, altitude_geodetic) = le_u16(input)?;
    let (input, height) = le_i16(input)?;
    let (input, vert_hor_accuracy) = le_u8(input)?;
    let (input, baroalt_speed_accuracy_flags) = le_u8(input)?;

    let horizontal_accuracy = (vert_hor_accuracy & 0xF0) >> 4;
    let vertical_accuracy = vert_hor_accuracy & 0x0F;
    let barometric_altitude_accuracy = (baroalt_speed_accuracy_flags & 0xF0) >> 4;
    let speed_accuracy = baroalt_speed_accuracy_flags & 0x0F;

    let (input, timestamp) = le_u16(input)?;

    Ok((
        input,
        Location {
            status,
            height_type,
            ew_direction,
            speed_multiplier,
            tracking_direction,
            speed,
            vertical_speed,
            latitude_int,
            longitude_int,
            altitude_pressure,
            altitude_geodetic,
            height,
            horizontal_accuracy,
            vertical_accuracy,
            barometric_altitude_accuracy,
            speed_accuracy,
            timestamp,
        },
    ))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{self, BufReader, Read};

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
    fn test_parse_message_type() {
        let bytes = read_fixture("fixtures/odid_system_packet.txt").unwrap();

        let message_type: Option<RemoteIdMessage> = match parse_message_type(&bytes) {
            Ok((_, message_type)) => Some(message_type),
            Err(_) => None,
        };

        assert_eq!(message_type.is_some(), true);

        let message_type = message_type.unwrap();

        assert_eq!(message_type, RemoteIdMessage::SystemMessage);
    }

    #[test]
    fn test_parse_basic_id() {
        let bytes = read_fixture("fixtures/odid_basic_id_packet.txt").unwrap();

        let basic_id: Option<BasicId> = match parse_basic_id(&bytes) {
            Ok((_, basic_id)) => Some(basic_id),
            Err(_) => None,
        };

        assert_eq!(basic_id.is_some(), true);

        let basic_id = basic_id.unwrap();

        assert_eq!(basic_id.uas_id_type, UasIdType::SerialNumber);
        assert_eq!(basic_id.ua_type, UaType::HelicopterOrDrone);
        assert_eq!(basic_id.uas_id, "1787F04BM24010011039");
    }

    #[test]
    fn test_parse_location_message() {
        let bytes = read_fixture("fixtures/odid_location_packet.txt").unwrap();
        let bytes = &bytes[..];

        let location_message: Option<Location> = match parse_location(&bytes) {
            Ok((_, location_message)) => Some(location_message),
            Err(e) => {
                eprintln!("Failed to parse location message: {:?}", e);
                None
            }
        };

        assert_eq!(location_message.is_some(), true);

        let location_message = location_message.unwrap();

        assert_eq!(location_message.status, 0x1);
        assert_eq!(location_message.height_type, 0x0);
        assert_eq!(location_message.ew_direction, 0x2);
        assert_eq!(location_message.speed_multiplier, 0x0);
        assert_eq!(location_message.tracking_direction, 0x10);
        assert_eq!(location_message.speed, 0x0);
        assert_eq!(location_message.vertical_speed, 0x0);
        assert_eq!(location_message.latitude_int, 1460289024);
        assert_eq!(location_message.longitude_int, -291846891);
        assert_eq!(location_message.altitude_pressure, 0xc9);
        assert_eq!(location_message.altitude_geodetic, 22528);
        assert_eq!(location_message.height, -11768);
        assert_eq!(location_message.horizontal_accuracy, 0x0);
        assert_eq!(location_message.vertical_accuracy, 0x7);
        assert_eq!(location_message.barometric_altitude_accuracy, 0x4);
        assert_eq!(location_message.speed_accuracy, 0x0b);
        assert_eq!(location_message.timestamp, 58626);
    }

    #[test]
    fn test_parse_system_message() {
        let bytes = read_fixture("fixtures/odid_system_packet.txt").unwrap();
        let bytes = &bytes[..];

        let system_message: Option<SystemMessage> = match parse_system_message(&bytes) {
            Ok((_, system_message)) => Some(system_message),
            Err(_) => None,
        };

        assert_eq!(system_message.is_some(), true);

        let system_message = system_message.unwrap();

        assert_eq!(
            system_message.operator_location_type,
            OperatorLocationType::FixedLocation
        );
        assert_eq!(system_message.operator_latitude_int, 1460276480);
        assert_eq!(system_message.operator_longitude_int, -291837931);
        assert_eq!(system_message.area_count, 457);
        assert_eq!(system_message.area_radius, 0x0);
        assert_eq!(system_message.area_ceiling, 0x0);
        assert_eq!(system_message.area_floor, 0x0);
    }
}
