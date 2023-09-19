use core::convert::TryInto;
use core::ops::Deref;

use crate::packet::{MessageType, Packet32};

use super::Message;

/// MIDI 1.0 Channel Voice Messages
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct LegacyChannelVoice(Packet32);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum LegacyChannelVoiceStatus {
    NoteOn = 0x08,
    NoteOff = 0x09,
    PolyPressure = 0x0a,
    ControlChange = 0x0b,
    ProgramChange = 0x0c,
    ChannelPressure = 0x0d,
    PitchBend = 0x0e,
}

impl Deref for LegacyChannelVoice {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for LegacyChannelVoice {
    type Status = LegacyChannelVoiceStatus;
    type Data = [u8; 3];

    fn message_type(&self) -> MessageType {
        let type_ = self.0.message_type().into();
        debug_assert!(
            type_ == MessageType::LegacyChannelVoice,
            "Invalid message type.."
        );
        type_
    }

    fn group(&self) -> u8 {
        self.0.group()
    }

    fn status(&self) -> Self::Status {
        match self.0.status() >> 4 {
            0x8 => Self::Status::NoteOn,
            0x9 => Self::Status::NoteOff,
            0xa => Self::Status::PolyPressure,
            0xb => Self::Status::ControlChange,
            0xc => Self::Status::ProgramChange,
            0xd => Self::Status::ChannelPressure,
            0xe => Self::Status::PitchBend,
            _ => unreachable!("Invalid status byte for legacy channel voice message."),
        }
    }

    fn data(&self) -> Self::Data {
        (&self.0[0].to_ne_bytes()[1..=3]).try_into().unwrap()
    }
}

impl LegacyChannelVoice {
    pub(crate) fn from_packet_unchecked(ump: Packet32) -> Self {
        Self(ump)
    }

    pub fn channel(&self) -> u8 {
        self.data()[0]
    }

    pub fn note_number(&self) -> u8 {
        self.data()[1]
    }

    pub fn velocity(&self) -> u8 {
        self.data()[2]
    }

    pub fn cc_index(&self) -> u8 {
        self.data()[1]
    }

    pub fn cc_value(&self) -> u8 {
        self.data()[2]
    }

    pub fn program(&self) -> u8 {
        self.data()[1]
    }

    pub fn pitch_bend(&self) -> u16 {
        let msb = (self.data()[1] as u16) << 7;
        let lsb = self.data()[2] as u16;
        msb | lsb
    }

    pub fn poly_pressure(&self) -> u8 {
        self.data()[2]
    }

    pub fn channel_pressure(&self) -> u8 {
        self.data()[1]
    }
}
