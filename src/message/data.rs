use core::ops::Deref;

use crate::packet::{MessageType, Packet128, Packet64};

use super::Message;

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Data64(Packet64);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Data128(Packet128);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum DataStatus {
    SinglePacket = 0x0,
    Start = 0x1,
    Continue = 0x2,
    End = 0x3,
    MixedDataSetHeader = 0x8,
    MixedDataSetPayload = 0x9,
}

impl Data64 {
    pub(crate) fn from_packet_unchecked(ump: Packet64) -> Self {
        Self(ump)
    }
}

impl Deref for Data64 {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for Data64 {
    type Status = DataStatus;
    type Data = [u8; 6];

    fn message_type(&self) -> MessageType {
        let type_ = self.0.message_type().into();
        debug_assert!(type_ == MessageType::Data64, "Invalid message type..");
        type_
    }

    fn group(&self) -> u8 {
        self.0.group()
    }

    fn status(&self) -> Self::Status {
        match (self.0[0] >> 20) & 0xf {
            0x0 => Self::Status::SinglePacket,
            0x1 => Self::Status::Start,
            0x2 => Self::Status::Continue,
            0x3 => Self::Status::End,
            _ => unreachable!("Invalid status byte for 8 byte data message."),
        }
    }

    fn data(&self) -> Self::Data {
        let bytes = [self.0[0].to_ne_bytes(), self.0[1].to_ne_bytes()];
        [
            bytes[0][1],
            bytes[0][0],
            bytes[1][3],
            bytes[1][2],
            bytes[1][1],
            bytes[1][0],
        ]
    }
}

impl Deref for Data128 {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Data128 {
    pub fn stream_id(&self) -> u8 {
        self.data()[0]
    }

    pub fn byte_count(&self) -> u8 {
        self.0[0].to_ne_bytes()[1] & 0xF
    }

    pub fn mds_id(&self) -> u8 {
        self.byte_count()
    }

    pub(crate) fn from_packet_unchecked(ump: Packet128) -> Self {
        Self(ump)
    }
}

impl Message for Data128 {
    type Status = DataStatus;
    type Data = [u8; 14];

    fn message_type(&self) -> MessageType {
        let type_ = self.0.message_type().into();
        debug_assert!(type_ == MessageType::Data128, "Invalid message type..");
        type_
    }

    fn group(&self) -> u8 {
        self.0.group()
    }

    fn status(&self) -> Self::Status {
        match (self.0[0] >> 20) & 0xf {
            0x0 => Self::Status::SinglePacket,
            0x1 => Self::Status::Start,
            0x2 => Self::Status::Continue,
            0x3 => Self::Status::End,
            0x4 => Self::Status::MixedDataSetHeader,
            0x5 => Self::Status::MixedDataSetPayload,
            _ => unreachable!("Invalid status byte for 16 byte data message."),
        }
    }

    fn data(&self) -> Self::Data {
        let bytes = [
            self.0[0].to_ne_bytes(),
            self.0[1].to_ne_bytes(),
            self.0[2].to_ne_bytes(),
            self.0[3].to_ne_bytes()
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

