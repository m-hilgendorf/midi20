//! Legacy (MIDI 1.x) channel voice messages.
use core::convert::TryInto;
use core::ops::Deref;

use crate::message::Message;
use crate::packet::{MessageType, Packet32};

/// MIDI 1.0 Channel Voice Messages.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct LegacyChannelVoice(Packet32);

/// Indicates the status of a legacy (MIDI 1.x) channel voice message.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum LegacyChannelVoiceStatus {
    /// This message is a note on message.
    NoteOn = 0x8,

    /// This message is a note off message.
    NoteOff = 0x9,

    /// This message is a polyphonic key pressure message.
    PolyPressure = 0xa,

    /// This message is a control change message.
    ControlChange = 0xb,

    /// This message is a program change message.
    ProgramChange = 0xc,

    /// This message is a channel key pressure message.
    ChannelPressure = 0xd,

    /// This message is a pitch bend message.
    PitchBend = 0xe,
}

impl LegacyChannelVoice {
    pub(crate) fn from_packet_unchecked(ump: Packet32) -> Self {
        Self(ump)
    }

    /// The destination channel for this message.
    pub fn channel(&self) -> u8 {
        self.data()[0]
    }

    /// Note number data for NoteOn, NoteOff.
    pub fn note_number(&self) -> u8 {
        self.data()[1]
    }

    /// Velocity value data for NoteOn, Note Off. A NoteOn velocity of 0 is equivalent to NoteOff.
    pub fn velocity(&self) -> u8 {
        self.data()[2]
    }

    /// Control change index data.
    pub fn cc_index(&self) -> u8 {
        self.data()[1]
    }

    /// Control change value data.
    pub fn cc_value(&self) -> u8 {
        self.data()[2]
    }

    /// Program change value data.
    pub fn program_change(&self) -> u8 {
        self.data()[1]
    }

    /// Pitch bend value data.
    pub fn pitch_bend(&self) -> u16 {
        let msb = (self.data()[1] as u16) << 7;
        let lsb = self.data()[2] as u16;
        msb | lsb
    }

    /// Polyphonic key pressure value data.
    pub fn poly_pressure(&self) -> u8 {
        self.data()[2]
    }

    /// Channel key pressure value data.
    pub fn channel_pressure(&self) -> u8 {
        self.data()[1]
    }
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
