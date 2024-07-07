use nom::bytes::complete::take;
use nom::number::complete::{le_u16, le_u32, le_u8};
use nom::IResult;
use radiotap::Radiotap;
use std::convert::TryInto;

use crate::wifi::{ASDSTAN_OUI, WIFI_ALLIANCE_OUI};

use super::{
    WifiActionFrame as ActionFrame, WifiBeaconFrame, WifiOpenDroneIDMessage as OpenDroneIDMessage,
    WifiOpenDroneIDMessagePack as OpenDroneIDMessagePack,
    WifiServiceDescriptorAttribute as ServiceDescriptorAttribute,
};

pub fn parse_open_drone_id_message_pack(input: &[u8]) -> IResult<&[u8], OpenDroneIDMessagePack> {
    let (input, message_type_and_version_pack) = le_u8(input)?;
    let message_pack_type = message_type_and_version_pack >> 4;
    let version_pack = message_type_and_version_pack & 0x0F;

    let (input, single_msg_size) = le_u8(input)?;
    let (mut input, num_messages) = le_u8(input)?;

    println!("message_pack_type: {:?}", message_type_and_version_pack);
    println!("single_msg_size: {:?}", single_msg_size);
    println!("num_messages: {:?}", num_messages);

    let mut messages = Vec::new();

    for _ in 1..=num_messages {
        let new_input = input;

        let (new_input, message_type_and_version) = le_u8(new_input)?;
        let message_type = message_type_and_version >> 4;
        let version = message_type_and_version & 0x0F;

        let (new_input, message_body) = take(24usize)(new_input)?;

        messages.push(OpenDroneIDMessage {
            message_type,
            version,
            message_body: message_body.try_into().unwrap(),
        });

        input = new_input;
    }

    Ok((
        input,
        OpenDroneIDMessagePack {
            message_type: message_pack_type,
            version: version_pack,
            single_msg_size,
            num_messages,
            messages,
        },
    ))
}

pub fn parse_service_descriptor_attribute(
    input: &[u8],
) -> IResult<&[u8], ServiceDescriptorAttribute> {
    let (input, attribute_id) = le_u8(input)?;
    let (input, attribute_length) = le_u16(input)?;
    let (input, service_id) = take(6usize)(input)?;
    let (input, instance_id) = le_u8(input)?;
    let (input, requestor_id) = le_u8(input)?;
    let (input, service_control) = le_u8(input)?;
    let (input, service_info_length) = le_u8(input)?;
    let (input, message_counter) = le_u8(input)?;

    if service_info_length == 0 || service_info_length == 1 {
        // create empty &[u8] if service_info_length is 0
        let service_info: &[u8] = &[];
        return Ok((
            input,
            ServiceDescriptorAttribute {
                attribute_id,
                attribute_length,
                service_id: service_id.try_into().unwrap(),
                instance_id,
                requestor_id,
                service_control,
                service_info_length,
                message_counter,
                service_info,
            },
        ));
    }

    if service_info_length < 1 {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }

    let (input, service_info) = take(service_info_length - 1)(input)?;

    Ok((
        input,
        ServiceDescriptorAttribute {
            attribute_id,
            attribute_length,
            service_id: service_id.try_into().unwrap(),
            instance_id,
            requestor_id,
            service_control,
            service_info_length,
            message_counter,
            service_info,
        },
    ))
}

pub fn parse_action_frame(input: &[u8]) -> IResult<&[u8], ActionFrame> {
    let (input, frame_control) = le_u16(input)?;
    let frame_control_version = (frame_control & 0b00000011) as u8;
    let frame_control_type = ((frame_control & 0b00001100) >> 2) as u8;
    let frame_control_subtype = ((frame_control & 0b11110000) >> 4) as u8;
    let (input, duration_id) = le_u16(input)?;
    let (input, address1) = take(6usize)(input)?;
    let (input, address2) = take(6usize)(input)?;
    let (input, address3) = take(6usize)(input)?;
    let (input, sequence_control) = le_u16(input)?;
    let (input, category) = take(1usize)(input)?;
    let (input, action) = take(1usize)(input)?;
    let (input, oui) = take(3usize)(input)?;
    let (input, oui_type) = take(1usize)(input)?;
    let (input, body) = take(input.len())(input)?;

    Ok((
        input,
        ActionFrame {
            frame_control,
            frame_control_version,
            frame_control_type,
            frame_control_subtype,
            duration_id,
            address1,
            address2,
            address3,
            sequence_control,
            category: category[0],
            action: action[0],
            oui: oui.try_into().unwrap(),
            oui_type: oui_type[0],
            body,
        },
    ))
}

pub fn parse_beacon_frame(input: &[u8]) -> IResult<&[u8], WifiBeaconFrame> {
    let (input, frame_control) = le_u16(input)?;
    let (input, duration) = le_u16(input)?;
    let (input, destination_addr) = take(6usize)(input)?;
    let (input, source_addr) = take(6usize)(input)?;
    let (input, bssid) = take(6usize)(input)?;
    let (input, sequence_control) = le_u16(input)?;
    let (input, _fixed_parameters) = take(12usize)(input)?;

    let mut tagged_parameters = input;
    let mut vendor_specific_data: &[u8] = &[];

    while !tagged_parameters.is_empty() {
        let (new_input, tag_number) = le_u8(tagged_parameters)?;

        if tag_number == 0xdd {
            let (new_input, tag_length) = le_u8(new_input)?;
            let (new_input, tag_oui) = take(3usize)(new_input)?;

            if tag_oui == WIFI_ALLIANCE_OUI || tag_oui == ASDSTAN_OUI {
                let (new_input, _tag_oui_type) = le_u8(new_input)?;
                let (new_input, _message_counter) = le_u8(new_input)?;

                let (_, tag_data) = take(tag_length as usize - 4)(new_input)?;

                vendor_specific_data = tag_data;
                tagged_parameters = new_input;
                break;
            }
        }

        tagged_parameters = new_input;
    }

    Ok((
        tagged_parameters,
        WifiBeaconFrame {
            frame_control,
            duration,
            destination_address: destination_addr.try_into().unwrap(),
            source_address: source_addr.try_into().unwrap(),
            bssid: bssid.try_into().unwrap(),
            sequence_control,
            vendor_specific_data,
        },
    ))
}

pub fn remove_radiotap_header(input: &[u8]) -> &[u8] {
    let radiotap: Option<Radiotap> = match Radiotap::from_bytes(input) {
        Ok(radiotap) => Some(radiotap),
        Err(error) => {
            println!(
                "Couldn't read packet data with Radiotap: {:?}, error {error:?}",
                &input
            );
            None
        }
    };

    if let Some(radiotap) = radiotap {
        &input[radiotap.header.length..]
    } else {
        input
    }
}
