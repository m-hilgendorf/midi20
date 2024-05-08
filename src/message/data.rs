#![allow(missing_docs)]
//! Data messages.
use core::convert::TryInto;
use core::ops::Deref;

use crate::message::Message;
use crate::packet::{MessageType, Packet128, Packet64};

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Data64(pub(crate) Packet64);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Data128(pub(crate) Packet128);

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
        unsafe {
            let data = self.as_ptr().cast::<u8>().add(2);
            core::slice::from_raw_parts(data, 6).try_into().unwrap()
        }
    }
}

impl Deref for Data128 {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Data128 {
    pub(crate) fn from_packet_unchecked(ump: Packet128) -> Self {
        Self(ump)
    }
}

impl Message for Data128 {
    type Status = DataStatus;
    type Data = [u8; 12];

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
            0x8 => Self::Status::MixedDataSetHeader,
            0x9 => Self::Status::MixedDataSetPayload,
            _ => unreachable!("Invalid status byte for 16 byte data message."),
        }
    }

    fn data(&self) -> Self::Data {
        unsafe {
            let data = self[1..3].as_ptr().cast();
            core::slice::from_raw_parts(data, 12).try_into().unwrap()
        }
    }
}

/// The `form` nibble of a data packet indicates its position within a stream.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DataFormat {
    /// Data is contained in a single packet.
    SinglePacket = 0,

    /// This is the start of a stream of data.
    Start = 1,

    /// This is a message in a stream of data.
    Continue = 2,

    /// This is a the final message in a stream of data.
    End = 3,

    /// Reserved.
    Reserved,
}

impl From<u8> for DataFormat {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Self::SinglePacket,
            0x1 => Self::Start,
            0x2 => Self::Continue,
            0x3 => Self::End,
            _ => Self::Reserved,
        }
    }
}

impl From<DataFormat> for u8 {
    fn from(value: DataFormat) -> Self {
        match value {
            DataFormat::SinglePacket => 0x0,
            DataFormat::Start => 0x1,
            DataFormat::Continue => 0x2,
            DataFormat::End => 0x3,
            DataFormat::Reserved => unreachable!(),
        }
    }
}
