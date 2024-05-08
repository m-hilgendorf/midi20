//! Utility message.
use core::ops::Deref;

use crate::message::Message;
use crate::packet::{MessageType, Packet, Packet32};

/// Utility messages defined by the MIDI 2.0 specification, including
/// jitter reduction and no-op.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Utility(pub(crate) Packet32);

/// Represents the status byte of a [Utility] message.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum UtilityStatus {
    /// The NoOp message. Data bits must be zeroed.
    NoOp = 0,

    /// A Jitter reduction timestamp message.
    JrTimestamp = 1,

    /// A Jitter reduction clock message.
    JrClock = 2,

    /// Declares the unit of mesaure used by [UtilityStatus::DeltaClockstamp] messages.
    DataClockstampTicksPerQuarternote = 3,

    /// Declares the time of all following messages which occur before the next delta clockstamp message.
    DeltaClockstamp = 4,
}

impl Utility {
    pub(crate) fn from_packet_unchecked(ump: Packet32) -> Self {
        Self(ump)
    }

    /// Create a new jitter reduction timestamp message.
    pub fn jr_timestamp(timestamp: u16) -> Self {
        Self(Packet([0x0020_0000 | (timestamp as u32)]))
    }

    /// Create a new jitter reduction clock message.
    pub fn jr_clock(clock: u16) -> Self {
        Self::from_packet_unchecked(Packet([0x0020_0000 | (clock as u32)]))
    }

    /// Create a new delta clockstamp message.
    pub fn delta_clockstamp(clock: u16) -> Self {
        Self::from_packet_unchecked(Packet([0x0020_0000 | (clock as u32)]))
    }

    /// Create a new delta clockstamp message in ticks per quarternote.
    pub fn delta_clockstamp_ticks_per_quarternote(clock: u16) -> Self {
        Self::from_packet_unchecked(Packet([0x0020_0000 | (clock as u32)]))
    }
}

impl Deref for Utility {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for Utility {
    type Data = u16;
    type Status = UtilityStatus;

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
            1 => Self::Status::JrTimestamp,
            2 => Self::Status::JrClock,
            3 => Self::Status::DataClockstampTicksPerQuarternote,
            4 => Self::Status::DeltaClockstamp,
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
