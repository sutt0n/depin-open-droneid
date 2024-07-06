use nom::error::Error;
use nom::number::complete::le_u16;
use nom::IResult;

// Constants for frame control field values
const BEACON_FRAME_SUBTYPE: u16 = 0b1000 << 4;
const ACTION_FRAME_SUBTYPE: u16 = 0b1101 << 4;
const MANAGEMENT_FRAME_TYPE: u16 = 0b00 << 2;

// Function to check if a frame is a beacon frame
pub fn is_beacon_frame(input: &[u8]) -> bool {
    let input = &input.to_owned()[..];
    let (_, frame_control) = le_u16::<&[u8], Error<&[u8]>>(input).unwrap_or_default();

    (frame_control & 0xFC) == (MANAGEMENT_FRAME_TYPE | BEACON_FRAME_SUBTYPE)
}

// Function to check if a frame is an action frame
pub fn is_action_frame(input: &[u8]) -> bool {
    let input = &input.to_owned()[..];
    let (_, frame_control) = le_u16::<&[u8], Error<&[u8]>>(input).unwrap_or_default();

    (frame_control & 0xFC) == (MANAGEMENT_FRAME_TYPE | ACTION_FRAME_SUBTYPE)
}
