use crate::messages::{BasicId, Location, Authentication, UaType, UasIdType};
use byteorder::{ByteOrder, LittleEndian, BigEndian};

pub fn parse_basic_id(data: &[u8]) -> BasicId {
    let id_type = (data[0] & 0xF0) >> 4;
    let ua_type = data[0] & 0x0F;

    // offset 1 -> length
    let uas_id = &data[1..data.len() - 1];

    // remove null bytes from uas_id
    let uas_id = uas_id.iter().cloned().filter(|&x| x != 0).collect::<Vec<u8>>();

    BasicId {
        uas_id_type: match id_type {
            0 => UasIdType::SerialNumber,
            1 => UasIdType::CaaRegistration,
            3 => UasIdType::UtmId,
            _ => UasIdType::Other(id_type),
        },
        ua_type: match ua_type {
            0 => UaType::Undeclared,
            1 => UaType::Aeroplane,
            2 => UaType::HelicopterOrDrone,
            3 => UaType::Gyroplane,
            4 => UaType::HybridLift,
            5 => UaType::Ornithopter,
            6 => UaType::Glider,
            7 => UaType::Kite,
            8 => UaType::FreeBalloon,
            9 => UaType::CaptiveBalloon,
            10 => UaType::Airship,
            11 => UaType::FreeFallParachute,
            12 => UaType::Rocket,
            13 => UaType::TetheredAircraft,
            14 => UaType::GroundObstacle,
            _ => UaType::Other(ua_type),
        },
        uas_id: String::from_utf8_lossy(&uas_id).to_string(),
    }
}

pub fn parse_location(data: &[u8]) -> Location {
    let status = (data[0] & 0xF0) >> 4;
    let reserved = data[0] & 0x08;
    let height_type = data[0] & 0x04;
    let ew_direction = data[0] & 0x02;
    let speed_multiplier = data[0] & 0x01;

    println!("Status: {} Reserved: {} Height Type: {} EW Direction: {} Speed Multiplier: {}", status, reserved, height_type, ew_direction, speed_multiplier);

    println!("Data: {:?} {}", data, data.len());

    Location {
        status,
        height_type,
        ew_direction,
        tracking_direction: data[1],
        speed: data[2],
        vertical_speed: data[3],
        latitude: LittleEndian::read_f32(&data[4..8]),
        // set every field to 0 for testing
        longitude: 0.0,
        altitude_pressure: 0,
        altitude_geodetic: 0,
        height: 0,
        horizontal_accuracy: 0,
        vertical_accuracy: 0,
        barometric_pressure_accuracy: 0,
        speed_accuracy: 0,
        timestamp: 0,
        // longitude: LittleEndian::read_f32(&data[9..13]),
        // altitude_pressure: LittleEndian::read_i16(&data[13..15]),
        // altitude_geodetic: LittleEndian::read_i16(&data[15..17]),
        // height: LittleEndian::read_i16(&data[17..19]),
        // horizontal_accuracy: (data[20] & 0xF0) >> 4,
        // vertical_accuracy: data[20] & 0x0F,
        // barometric_pressure_accuracy: data[21] & 0xF0 >> 4,
        // speed_accuracy: data[21] & 0x0F,
        // timestamp: LittleEndian::read_u24(&data[22..23]),
    }
}

pub fn parse_authentication(data: &[u8]) -> Authentication {
    Authentication {
        auth_type: data[0],
        page: data[1],
        length: data[2],
        timestamp: LittleEndian::read_u24(&data[3..6]),
        auth_data: data[6..].to_vec(),
    }
}
