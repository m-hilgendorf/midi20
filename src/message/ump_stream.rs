#![allow(missing_docs)]
use core::ops::Deref;

use crate::message::{data::DataFormat, Message};
use crate::packet::{MessageType, Packet128};

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct UmpStream(pub(crate) Packet128);

impl Deref for UmpStream {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for UmpStream {
    type Data = [u8; 14];
    type Status = UmpStreamStatus;

    fn message_type(&self) -> MessageType {
        let msg_type = self.0.message_type().into();
        debug_assert!(msg_type == MessageType::UmpStream, "Invalid message type");
        msg_type
    }

    /// UMP Stream messages are group-less.
    fn group(&self) -> u8 {
        0
    }

    fn status(&self) -> Self::Status {
        let bytes = self.0[0].to_ne_bytes();
        bytes[1].into()
    }

    fn data(&self) -> Self::Data {
        let bytes = [
            self.0[0].to_ne_bytes(),
            self.0[1].to_ne_bytes(),
            self.0[2].to_ne_bytes(),
            self.0[3].to_ne_bytes(),
        ];
        [
            bytes[0][1],
            bytes[0][0],
            bytes[1][3],
            bytes[1][2],
            bytes[1][1],
            bytes[1][0],
            bytes[2][3],
            bytes[2][2],
            bytes[2][1],
            bytes[2][0],
            bytes[3][3],
            bytes[3][2],
            bytes[3][1],
            bytes[3][0],
        ]
    }
}

impl UmpStream {
    pub(crate) fn from_packet_unchecked(packet: Packet128) -> Self {
        Self(packet)
    }

    pub fn format(&self) -> DataFormat {
        let bytes = self.0[0].to_ne_bytes();
        (bytes[0] & 0xF).into()
    }
}

pub enum UmpStreamStatus {
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

impl From<u8> for UmpStreamStatus {
    fn from(value: u8) -> Self {
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

impl UmpStream {
    pub fn endpoint_discovery(&self) -> EndpointDiscovery {
        EndpointDiscovery {
            ump_major_version: self.data()[0],
            ump_minor_version: self.data()[1],
            filter_bitmap: self.data()[5],
        }
    }

    pub fn endpoint_info_notification(&self) -> EndpointInfoNotification {
        EndpointInfoNotification {
            ump_major_version: self.data()[0],
            ump_minor_version: self.data()[1],
            function_block_count: self.data()[2],
            m1_support: self.data()[3],
            m2_support: self.data()[4],
            jitter_reduction_support: self.data()[5],
        }
    }

    pub fn endpoint_name_notification(&self) -> EndpointNameIdentification {
        EndpointNameIdentification(self.data())
    }

    pub fn product_instance_id_notification(&self) -> ProductInstanceIdNotification {
        ProductInstanceIdNotification(self.data())
    }

    pub fn stream_configuration_request(&self) -> StreamConfigurationRequest {
        StreamConfigurationRequest {
            protocol: self.data()[0],
            jitter_reduction: self.data()[1],
        }
    }

    pub fn stream_configuration_notification(&self) -> StreamConfigurationNotification {
        StreamConfigurationNotification {
            protocol: self.data()[0],
            jitter_reduction: self.data()[1],
        }
    }

    pub fn function_block_discovery(&self) -> FunctionBlockDiscovery {
        FunctionBlockDiscovery {
            function_block_count: self.data()[0],
            filter_bitmap: self.data()[1],
        }
    }

    pub fn function_block_info_notification(&self) -> FunctionBlockInfoNotification {
        FunctionBlockInfoNotification {
            function_block_count: self.data()[0],
            function_block_data: [
                self.data()[1],
                self.data()[2],
                self.data()[3],
                self.data()[4],
                self.data()[5],
            ],
        }
    }

    pub fn function_block_name_notification(&self) -> FunctionBlockNameNotification {
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
