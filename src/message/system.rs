use core::ops::Deref;

use crate::packet::{MessageType, Packet32};

use super::Message;

/// System common messages, for time code, song position, song select, and tune request
/// Note: all data is 8 bit except for SongSelect.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct System(Packet32);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SystemStatus {
    TimeCode = 0xf1,
    SongPositionPointer = 0xf2,
    SongSelect = 0xf3,
    TuneRequest = 0xf6,
    TimingClock = 0xf8,
    Start = 0xfa,
    Continue = 0xfb,
    Stop = 0xfc,
    ActiveSensing = 0xfe,
    Reset = 0xff,
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
    type Status = SystemStatus;
    type Data = [u8; 2];

    fn message_type(&self) -> MessageType {
        let type_ = self.0.message_type().into();
        debug_assert!(type_ == MessageType::System, "Invalid message type..");
        type_
    }

    fn group(&self) -> u8 {
        self.0.group()
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
        use core::convert::TryInto;

        self.0[0].to_be_bytes()[2..=3].try_into().unwrap()
    }
}

impl System {
    pub fn time_code(&self) -> u8 {
        self.data()[0]
    }

    pub fn song_select(&self) -> u8 {
        self.data()[0]
    }

    pub fn song_position_pointer(&self) -> u16 {
        u16::from_be_bytes(self.data())
    }

    pub(crate) fn from_packet_unchecked(ump: Packet32) -> Self {
        Self(ump)
    }
}
