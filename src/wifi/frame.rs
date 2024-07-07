// Function to check if a frame is a beacon frame
pub fn is_beacon_frame(input: &[u8]) -> bool {
    if input[0] == 0x80 {
        return true;
    }

    false
}

pub fn is_action_frame(input: &[u8]) -> bool {
    if input[0] == 0xd0 {
        return true;
    }

    false
}
