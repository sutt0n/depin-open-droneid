use crate::messages::{
    Authentication, BasicId, Location, Operator, OperatorLocationType, SystemMessage, UaType,
    UasIdType,
};
use byteorder::{BigEndian, ByteOrder, LittleEndian};

pub fn parse_basic_id(data: &[u8]) -> BasicId {
    let id_type = (data[0] & 0xF0) >> 4;
    let ua_type = data[0] & 0x0F;

    // offset 1 -> length
    let uas_id = &data[1..data.len() - 1];

    // remove null bytes from uas_id
    let uas_id = uas_id
        .iter()
        .cloned()
        .filter(|&x| x != 0)
        .collect::<Vec<u8>>();

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

pub fn parse_system_message(data: &[u8]) -> SystemMessage {
    let flags = &data[0];

    // operator_location_type: flag bits 0 - 1
    let operator_location_type = flags & 0x03;

    let operator_location_type = match operator_location_type {
        0 => OperatorLocationType::TakeOff,
        1 => OperatorLocationType::LiveGNSS,
        2 => OperatorLocationType::FixedLocation,
        i => OperatorLocationType::Other(i),
    };

    SystemMessage {
        operator_location_type,
        operator_latitude_int: LittleEndian::read_i32(&data[1..5]),
        operator_longitude_int: LittleEndian::read_i32(&data[5..9]),
        area_count: LittleEndian::read_i16(&data[9..11]),
        area_radius: data[12],
        area_ceiling: LittleEndian::read_u16(&data[13..15]),
        area_floor: LittleEndian::read_u16(&data[15..17]),
    }
}

pub fn parse_operator_id(data: &[u8]) -> Operator {
    let operator_id_type = data[0];
    let operator_id = &data[1..data.len() - 3]
        .iter()
        .cloned()
        .filter(|&x| x != 0)
        .collect::<Vec<u8>>();

    Operator {
        operator_id_type,
        operator_id: String::from_utf8_lossy(&operator_id).to_string(),
    }
}

pub fn parse_location(data: &[u8]) -> Location {
    let status = (data[0] & 0xF0) >> 4;
    let reserved = data[0] & 0x08;
    let height_type = data[0] & 0x04;
    let ew_direction = data[0] & 0x02;
    let speed_multiplier = data[0] & 0x01;

    Location {
        status,
        height_type,
        ew_direction,
        speed_multiplier,
        tracking_direction: data[1],
        speed: data[2],
        vertical_speed: data[3],
        // latitude is 4 bytes from bit 3 to 7
        latitude_int: LittleEndian::read_i32(&data[4..8]),
        // set every field to 0 for testing
        longitude_int: LittleEndian::read_i32(&data[8..12]),
        altitude_pressure: LittleEndian::read_u16(&data[12..14]),
        altitude_geodetic: LittleEndian::read_u16(&data[14..16]),
        height: LittleEndian::read_i16(&data[16..18]),
        horizontal_accuracy: (data[18] & 0xF0) >> 4,
        vertical_accuracy: data[18] & 0x0F,
        barometric_altitude_accuracy: (data[19] & 0xF0) >> 4,
        speed_accuracy: data[19] & 0x0F,
        timestamp: LittleEndian::read_u16(&data[19..21]),
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
