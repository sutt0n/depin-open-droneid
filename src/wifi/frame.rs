// Function to check if a frame is a beacon frame
pub async fn is_beacon_frame(input: &[u8], header_length: u8) -> bool {
    if input[header_length as usize] == 0x80 {
        return true;
    }

    false
}

pub async fn is_action_frame(input: &[u8], header_length: u8) -> bool {
    if input[header_length as usize] == 0xd0 {
        return true;
    }

    false
}
