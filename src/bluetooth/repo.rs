use nom::IResult;
use nom::bytes::complete::take;
use nom::number::complete::le_u8;

use super::BluetoothAdvertisementFrame;

pub fn parse_bluetooth_advertisement_frame(input: &[u8]) -> IResult<&[u8], BluetoothAdvertisementFrame> {
    let (input, app_code) = le_u8(input)?;
    let (input, counter) = le_u8(input)?;
    let (input, message) = take(25usize)(input)?;

    Ok((
        input,
        BluetoothAdvertisementFrame {
            app_code,
            counter,
            message: message.try_into().unwrap(),
        },
    ))
}

#[cfg(test)]
pub mod test {
    use crate::odid::{parse_location, Location};

    use super::*;
    use std::fs::File;
    use std::io::{self, Read, BufReader};

    fn read_fixture(file_path: &str) -> io::Result<Vec<u8>> {
        // Open the file
        let file = File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);
    
        // Read the file content into a string
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;
    
        // Trim the square brackets and split the string by comma
        let content = content.trim().trim_start_matches('[').trim_end_matches(']');
        let bytes: Vec<u8> = content
            .split(',')
            .map(|s| s.trim().parse().expect("Failed to parse byte"))
            .collect();
    
        Ok(bytes)
    }

    #[test]
    fn test_parse_bluetooth_advertisement_frame() {
        let input = read_fixture("fixtures/bluetooth_location_packet.txt").unwrap();

        let bt_advertisement_frame: Option<BluetoothAdvertisementFrame> = match parse_bluetooth_advertisement_frame(&input) {
            Ok((_, frame)) => Some(frame),
            Err(e) => {
                eprintln!("Failed to parse Bluetooth advertisement frame: {:?}", e);
                None
            }
        };

        assert_eq!(bt_advertisement_frame.is_some(), true);

        let bt_advertisement_frame = bt_advertisement_frame.unwrap();

        assert_eq!(bt_advertisement_frame.app_code, 0x0d);
        assert_eq!(bt_advertisement_frame.counter, 33);
        assert_eq!(bt_advertisement_frame.message.len(), 25);

        let location: Option<Location> = match parse_location(&bt_advertisement_frame.message) {
            Ok((_, location)) => Some(location),
            Err(_) => None,
        };

        assert_eq!(location.is_some(), true);

        let location = location.unwrap();

        assert_eq!(location.latitude_int, 1460289024);
        assert_eq!(location.longitude_int, -291846891);

    }
}
