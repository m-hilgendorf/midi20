use core::ops::Deref;

use crate::message::Message;
use crate::packet::{MessageType, Packet32};

/// Utility messages defined by the MIDI 2.0 specification, including
/// jitter reduction and no-op.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Utility(Packet32);

/// Represents the status byte of a [Utility] message.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum UtilityStatus {
    NoOp = 0,
    JitterReduction = 1,
    JitterReductionClock = 2,
}

impl Utility {
    pub(crate) fn from_packet_unchecked(ump: Packet32) -> Self {
        Self(ump)
    }
}

impl Deref for Utility {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for Utility {
    type Status = UtilityStatus;
    type Data = u16;

    fn message_type(&self) -> MessageType {
        let type_ = self.0.message_type().into();
        debug_assert!(type_ == MessageType::Utility, "Invalid message type");
        type_
    }

    fn group(&self) -> u8 {
        self.0.group()
    }

    fn status(&self) -> Self::Status {
        let status = self.0.status();
        match status {
            0 => Self::Status::NoOp,
            1 => Self::Status::JitterReduction,
            2 => Self::Status::JitterReductionClock,
            _ => unreachable!("Invalid status byte for utility message."),
        }
    }

    fn data(&self) -> u16 {
        (self.0[0] & 0x0000_ffff) as u16
    }
}

impl From<Utility> for Packet32 {
    fn from(value: Utility) -> Packet32 {
        value.0
    }
}
