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
    use super::*;

    #[test]
    fn test_parse_bluetooth_advertisement_frame() {
        let input = [
            13,  // app code
            77,  // message counter
            2, 18, 49, 55, 56, 55, 70, 48, 52, 66, 77, 50, 52, 48, 49, 48, 48, 49, 49, 48, 51, 57, 0, 0, 0
        ];

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
        assert_eq!(bt_advertisement_frame.counter, 0x4d);
    }
}
