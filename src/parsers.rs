use crate::messages::{BasicId, Location, Authentication, UaType, UasIdType};
use byteorder::{ByteOrder, LittleEndian};

pub fn parse_basic_id(data: &[u8]) -> BasicId {
    let id_type = (data[1] & 0xF0) >> 4;
    let ua_type = data[1] & 0x0F;
    let uas_id = data[2..21].to_vec();

    BasicId {
        uas_id_type: match id_type {
            0 => UasIdType::SerialNumber,
            1 => UasIdType::CaaRegistration,
            _ => UasIdType::Other(id_type),
        },
        ua_type: match ua_type {
            0 => UaType::Undeclared,
            1 => UaType::Aeroplane,
            2 => UaType::Helicopter,
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
    Location {
        status: data[0],
        direction: LittleEndian::read_u16(&data[1..3]),
        speed: LittleEndian::read_u16(&data[3..5]),
        latitude: LittleEndian::read_f32(&data[5..9]),
        longitude: LittleEndian::read_f32(&data[9..13]),
        altitude_pressure: LittleEndian::read_i16(&data[13..15]),
        altitude_geodetic: LittleEndian::read_i16(&data[15..17]),
        height: LittleEndian::read_i16(&data[17..19]),
        horizontal_accuracy: data[19],
        vertical_accuracy: data[20],
        barometric_pressure_accuracy: data[21],
        speed_accuracy: data[22],
        timestamp: LittleEndian::read_u24(&data[23..26]),
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
