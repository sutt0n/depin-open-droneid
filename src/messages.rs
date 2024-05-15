#[derive(Debug)]
pub enum UasIdType {
    SerialNumber,
    CaaRegistration,
    Other(u8),
}

#[derive(Debug)]
pub enum UaType {
    Undeclared,
    Aeroplane,
    Helicopter,
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

#[derive(Debug)]
pub struct BasicId {
    pub uas_id_type: UasIdType,
    pub ua_type: UaType,
    pub uas_id: String,  // Assuming UTF-8 encoding; adjust based on actual spec
}

#[derive(Debug)]
pub struct Location {
    pub status: u8,
    pub direction: u16,
    pub speed: u16,
    pub latitude: f32,
    pub longitude: f32,
    pub altitude_pressure: i16,
    pub altitude_geodetic: i16,
    pub height: i16,
    pub horizontal_accuracy: u8,
    pub vertical_accuracy: u8,
    pub barometric_pressure_accuracy: u8,
    pub speed_accuracy: u8,
    pub timestamp: u32,
}

#[derive(Debug)]
pub struct Authentication {
    pub auth_type: u8,
    pub page: u8,
    pub length: u8,
    pub timestamp: u32,
    pub auth_data: Vec<u8>,
}

#[derive(Debug)]
pub enum RemoteIdMessage {
    BasicId(BasicId),
    Location(Location),
    Authentication(Authentication),
    Unknown,
}

