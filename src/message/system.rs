//! System common and real-time messages.
use core::convert::TryInto;
use core::ops::Deref;

use crate::message::Message;
use crate::packet::{MessageType, Packet32};

/// System common messages, for time code, song position, song select, and tune request
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct System(pub(crate) Packet32);

/// Represents the status byte of a [System] message.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SystemStatus {
    /// MIDI time code. Data is the MIDI time code value.
    TimeCode = 0xf1,

    /// The song position pointer. Data is the position in the song.
    SongPositionPointer = 0xf2,

    /// A song selection message. Data is the song to be selected.
    SongSelect = 0xf3,

    /// A tune request message. Data is reserved.
    TuneRequest = 0xf6,

    /// A timing clock message. Data is reserved.
    TimingClock = 0xf8,

    /// A "start" transport message. Data is reserved.
    Start = 0xfa,

    /// A "continue" transport message. Data is reserved.
    Continue = 0xfb,

    /// A "stop" transport message. Data is reserved.
    Stop = 0xfc,

    /// Enable active sensing mode.
    ActiveSensing = 0xfe,

    /// Reset the device.
    Reset = 0xff,
}

impl System {
    pub(crate) fn from_packet_unchecked(ump: Packet32) -> Self {
        Self(ump)
    }

    /// Returns the time code data of this message. Note: the status must be [SystemStatus::TimeCode].
    pub fn time_code(&self) -> u8 {
        self.data()[0]
    }

    /// Returns the song selection data of this message. Note: the status must be [SystemStatus::SongSelect].
    pub fn song_select(&self) -> u8 {
        self.data()[0]
    }

    /// Returns the song position pointer of this message. Note: the status must be [SystemStatus::SongPositionPointer].
    pub fn song_position_pointer(&self) -> u16 {
        u16::from_be_bytes(self.data())
    }
}

impl From<System> for Packet32 {
    fn from(value: System) -> Packet32 {
        value.0
    }
}

impl Deref for System {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for System {
    type Data = [u8; 2];
    type Status = SystemStatus;

    fn group(&self) -> u8 {
        self.0.group()
    }

    fn message_type(&self) -> MessageType {
        let type_ = self.0.message_type().into();
        debug_assert!(type_ == MessageType::System, "Invalid message type..");
        type_
    }

    fn status(&self) -> Self::Status {
        let status = self.0.status();
        match status {
            0xf1 => Self::Status::TimeCode,
            0xf2 => Self::Status::SongPositionPointer,
            0xf3 => Self::Status::SongSelect,
            0xf6 => Self::Status::TuneRequest,
            0xf8 => Self::Status::TimingClock,
            0xfa => Self::Status::Start,
            0xfb => Self::Status::Continue,
            0xfc => Self::Status::Stop,
            0xfe => Self::Status::ActiveSensing,
            0xff => Self::Status::Reset,
            _ => unreachable!("Invalid status byte for system common message."),
        }
    }

    fn data(&self) -> Self::Data {
        self.0[0].to_be_bytes()[2..=3].try_into().unwrap()
    }
}
