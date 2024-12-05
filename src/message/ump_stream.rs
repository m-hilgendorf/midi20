#![allow(missing_docs)]
use core::convert::TryInto;
use core::ops::Deref;

use crate::message::{data::DataFormat, Message};
use crate::packet::{MessageType, Packet, Packet128};

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct UmpStream(pub(crate) Packet128);

impl Deref for UmpStream {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for UmpStream {
    type Data = [u32; 3];
    type Status = Status;

    fn message_type(&self) -> MessageType {
        let msg_type = self.0.message_type().into();
        debug_assert!(msg_type == MessageType::UmpStream, "Invalid message type");
        msg_type
    }

    /// UMP Stream messages are group-less.
    fn group(&self) -> u8 {
        debug_assert!(false, "ump stream messages have no group");
        0
    }

    fn status(&self) -> Self::Status {
        ((self.0[0] >> 16) as u16).into()
    }

    fn data(&self) -> Self::Data {
        self.0[1..3].try_into().unwrap()
    }
}

impl UmpStream {
    pub(crate) fn from_packet_unchecked(packet: Packet128) -> Self {
        Self(packet)
    }

    pub fn format(&self) -> DataFormat {
        let bytes = self.0[0].to_be_bytes();
        (bytes[0] & 0xF).into()
    }
}

pub enum Status {
    EndpointDiscovery,
    EndpointInfoNotification,
    DeviceIdentityNotification,
    EndpointNameNotification,
    ProductInstanceIdNotification,
    StreamConfigurationRequest,
    StreamConfigurationNotification,
    FunctionBlockDiscovery,
    FunctionBlockInfoNotification,
    FunctionBlockNameNotification,
    StartOfClip,
    EndOfClip,
    Reserved,
}

impl From<u16> for Status {
    fn from(value: u16) -> Self {
        match value {
            0x0 => Self::EndpointDiscovery,
            0x1 => Self::EndpointInfoNotification,
            0x2 => Self::DeviceIdentityNotification,
            0x3 => Self::EndpointNameNotification,
            0x4 => Self::ProductInstanceIdNotification,
            0x5 => Self::StreamConfigurationRequest,
            0x6 => Self::StreamConfigurationNotification,
            0x10 => Self::FunctionBlockDiscovery,
            0x11 => Self::FunctionBlockInfoNotification,
            0x12 => Self::FunctionBlockNameNotification,
            0x20 => Self::StartOfClip,
            0x21 => Self::EndOfClip,
            _ => Self::Reserved,
        }
    }
}

impl From<Status> for u8 {
    fn from(value: Status) -> u8 {
        match value {
            Status::EndpointDiscovery => 0x0,
            Status::EndpointInfoNotification => 0x1,
            Status::DeviceIdentityNotification => 0x2,
            Status::EndpointNameNotification => 0x3,
            Status::ProductInstanceIdNotification => 0x4,
            Status::StreamConfigurationRequest => 0x5,
            Status::StreamConfigurationNotification => 0x6,
            Status::FunctionBlockDiscovery => 0x10,
            Status::FunctionBlockInfoNotification => 0x11,
            Status::FunctionBlockNameNotification => 0x12,
            Status::StartOfClip => 0x20,
            Status::EndOfClip => 0x21,
            Status::Reserved => 0x22,
        }
    }
}

impl UmpStream {
    pub fn endpoint_discovery(data: EndpointDiscovery) -> Self {
        let word0 = u32::from_be_bytes([
            u8::from(Status::EndpointDiscovery) << 4,
            0,
            data.ump_major_version,
            data.ump_minor_version,
        ]);
        let word1 = u32::from_be_bytes([0, 0, 0, data.filter_bitmap]);
        let word2 = 0;
        let word3 = 0;
        Self::from_packet_unchecked(Packet([word0, word1, word2, word3]))
    }

    pub fn get_endpoint_discovery(&self) -> EndpointDiscovery {
        todo!()
    }

    pub fn get_endpoint_info_notification(&self) -> EndpointInfoNotification {
        todo!()
    }

    pub fn get_endpoint_name_notification(&self) -> EndpointNameIdentification {
        todo!()
    }

    pub fn get_product_instance_id_notification(&self) -> ProductInstanceIdNotification {
        todo!()
    }

    pub fn get_stream_configuration_request(&self) -> StreamConfigurationRequest {
        todo!()
    }

    pub fn get_stream_configuration_notification(&self) -> StreamConfigurationNotification {
        todo!()
    }

    pub fn get_function_block_discovery(&self) -> FunctionBlockDiscovery {
        todo!()
    }

    pub fn get_function_block_info_notification(&self) -> FunctionBlockInfoNotification {
        todo!()
    }

    pub fn get_function_block_name_notification(&self) -> FunctionBlockNameNotification {
        todo!()
    }
}

pub struct EndpointDiscovery {
    pub ump_major_version: u8,
    pub ump_minor_version: u8,
    pub filter_bitmap: u8,
}

pub struct EndpointInfoNotification {
    pub ump_major_version: u8,
    pub ump_minor_version: u8,
    pub function_block_count: u8,
    pub m1_support: u8,
    pub m2_support: u8,
    pub jitter_reduction_support: u8,
}

pub struct DeviceIdentityNotification {
    pub sysex_id: u8,
    pub family_id: u8,
    pub model_id: u8,
    pub version_id: u8,
}

pub struct EndpointNameIdentification(pub [u8; 14]);

pub struct ProductInstanceIdNotification(pub [u8; 14]);

pub struct StreamConfigurationRequest {
    pub protocol: u8,
    pub jitter_reduction: u8,
}

pub struct StreamConfigurationNotification {
    pub protocol: u8,
    pub jitter_reduction: u8,
}

pub struct FunctionBlockDiscovery {
    pub function_block_count: u8,
    pub filter_bitmap: u8,
}

pub struct FunctionBlockInfoNotification {
    pub function_block_count: u8,
    pub function_block_data: [u8; 5],
}

pub struct FunctionBlockNameNotification {
    pub function_block_count: u8,
    pub name_bytes: [u8; 12],
}
