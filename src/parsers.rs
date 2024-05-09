use crate::messages::{BasicId, Location, Authentication, UasIdType};
use byteorder::{ByteOrder, LittleEndian};

pub fn parse_basic_id(data: &[u8]) -> BasicId {
    BasicId {
        uas_id_type: match data[0] {
            0 => UasIdType::SerialNumber,
            1 => UasIdType::CaaRegistration,
            _ => UasIdType::Other(data[0]),
        },
        uas_id: String::from_utf8_lossy(&data[1..21]).to_string(),
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
