use crate::messages::{BasicId, Location, Authentication, UaType, UasIdType};
use byteorder::{ByteOrder, LittleEndian, BigEndian};

pub fn parse_basic_id(data: &[u8]) -> BasicId {
    let id_type = (data[1] & 0xF0) >> 4;
    let ua_type = data[1] & 0x0F;

    // offset 2, length 20
    let uas_id = data[2..22].to_vec();

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
        status: data[1],
        direction: LittleEndian::read_u16(&data[2..4]),
        speed: LittleEndian::read_u16(&data[4..6]),
        // Latitude of UA deg*10^7 Int signed (LE) 
        latitude: LittleEndian::read_f32(&data[6..10]) * 1e-7,
        // Longitude of UA deg*10^7 Int signed (LE)
        longitude: LittleEndian::read_f32(&data[10..14]) * 1e-7,
        altitude_pressure: LittleEndian::read_i16(&data[14..16]),
        altitude_geodetic: LittleEndian::read_i16(&data[16..18]),
        height: LittleEndian::read_i16(&data[18..20]),
        horizontal_accuracy: data[20],
        vertical_accuracy: data[21],
        barometric_pressure_accuracy: data[22],
        speed_accuracy: data[23],
        timestamp: LittleEndian::read_u24(&data[24..27]),
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
