use core::ops::Deref;

use crate::message::DataFormat;
use crate::packet::{MessageType, Packet128, Packet64};

use super::Message;

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Data64(Packet64);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Data128(Packet128);

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
    type Status = DataFormat;
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
        (((self.0[0] >> 20) & 0xf) as u8).into()
    }

    fn data(&self) -> Self::Data {
        let bytes = [self.0[0].to_ne_bytes(), self.0[1].to_ne_bytes()];
        [
            bytes[0][0],
            bytes[0][1],
            bytes[1][0],
            bytes[1][1],
            bytes[1][2],
            bytes[1][3],
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
    type Status = ExtendedDataFormat;
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
        (((self.0[0] >> 20) & 0xf) as u8).into()
    }

    fn data(&self) -> Self::Data {
        let bytes = [
            self.0[0].to_ne_bytes(),
            self.0[1].to_ne_bytes(),
            self.0[2].to_ne_bytes(),
            self.0[3].to_ne_bytes()
        ];
        [
            bytes[0][0],
            bytes[0][1],
            bytes[1][0],
            bytes[1][1],
            bytes[1][2],
            bytes[1][3],
            bytes[2][0],
            bytes[2][1],
            bytes[2][2],
            bytes[2][3],
            bytes[3][0],
            bytes[3][1],
            bytes[3][2],
            bytes[3][3],
        ]
    }
}


#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum ExtendedDataFormat {
    SinglePacket,
    Start,
    Continue,
    End,
    MixedDataSetHeader,
    MixedDataSetPayload,
    Reserved,
}

impl From<u8> for ExtendedDataFormat {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::SinglePacket,
            1 => Self::Start,
            2 => Self::Continue,
            3 => Self::End,
            8 => Self::MixedDataSetHeader,
            9 => Self::MixedDataSetPayload,
            _ => Self::Reserved,
        }
    }
}

impl From<ExtendedDataFormat> for u8 {
    fn from(value: ExtendedDataFormat) -> Self {
        match value {
            ExtendedDataFormat::SinglePacket => 0,
            ExtendedDataFormat::Start => 1,
            ExtendedDataFormat::Continue => 2,
            ExtendedDataFormat::End => 3,
            ExtendedDataFormat::MixedDataSetHeader => 8,
            ExtendedDataFormat::MixedDataSetPayload => 9,
            ExtendedDataFormat::Reserved => unreachable!()
        }
    }
}

