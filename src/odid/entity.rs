use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UasIdType {
    SerialNumber,
    CaaRegistration,
    UtmId,
    Other(u8),
}

impl From<u8> for UasIdType {
    fn from(value: u8) -> Self {
        match value {
            0 => UasIdType::SerialNumber,
            1 => UasIdType::CaaRegistration,
            2 => UasIdType::UtmId,
            _ => UasIdType::Other(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UaType {
    Undeclared,
    Aeroplane,
    HelicopterOrDrone,
    Gyroplane,
    HybridLift,
    Ornithopter,
    Glider,
    Kite,
    FreeBalloon,
    CaptiveBalloon,
    Airship,
    FreeFallParachute,
    Rocket,
    TetheredAircraft,
    GroundObstacle,
    Other(u8),
}

impl From<u8> for UaType {
    fn from(value: u8) -> Self {
        match value {
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
            _ => UaType::Other(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicId {
    pub uas_id_type: UasIdType,
    pub ua_type: UaType,
    pub uas_id: String, // Assuming UTF-8 encoding; adjust based on actual spec
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub status: u8,
    pub ew_direction: u8,
    pub height_type: u8,
    pub tracking_direction: u8,
    pub speed_multiplier: u8,
    pub speed: u8,
    pub vertical_speed: u8,
    pub latitude_int: i32,
    pub longitude_int: i32,
    pub altitude_pressure: u16,
    pub altitude_geodetic: u16,
    pub height: i16,
    pub horizontal_accuracy: u8,
    pub vertical_accuracy: u8,
    pub barometric_altitude_accuracy: u8,
    pub speed_accuracy: u8,
    pub timestamp: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authentication {
    pub auth_type: u8,
    pub page: u8,
    pub length: u8,
    pub timestamp: u32,
    pub auth_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperatorLocationType {
    TakeOff,
    LiveGNSS,
    FixedLocation,
    Other(u8),
}

impl From<u8> for OperatorLocationType {
    fn from(value: u8) -> Self {
        match value {
            0 => OperatorLocationType::TakeOff,
            1 => OperatorLocationType::LiveGNSS,
            2 => OperatorLocationType::FixedLocation,
            _ => OperatorLocationType::Other(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub operator_location_type: OperatorLocationType,
    pub operator_latitude_int: i32,
    pub operator_longitude_int: i32,
    pub area_count: i16,
    pub area_radius: u8,
    pub area_ceiling: u16,
    pub area_floor: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operator {
    pub operator_id_type: u8,
    pub operator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteIdMessage {
    BasicId,
    Location,
    Authentication,
    SelfId,
    SystemMessage,
    OperatorId,
    MessagePack,
    Unknown,
}

impl From<u8> for RemoteIdMessage {
    fn from(value: u8) -> Self {
        match value {
            0 => RemoteIdMessage::BasicId,
            1 => RemoteIdMessage::Location,
            2 => RemoteIdMessage::Authentication,
            3 => RemoteIdMessage::SelfId,
            4 => RemoteIdMessage::SystemMessage,
            5 => RemoteIdMessage::OperatorId,
            0xf => RemoteIdMessage::MessagePack,
            _ => RemoteIdMessage::Unknown,
        }
    }
}
