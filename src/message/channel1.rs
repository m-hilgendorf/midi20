//! Legacy (MIDI 1.x) channel voice messages.
use core::convert::TryInto;
use core::ops::Deref;

use crate::message::Message;
use crate::packet::{MessageType, Packet, Packet32};

use super::channel2::ChannelVoice;

/// MIDI 1.0 Channel Voice Messages.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct LegacyChannelVoice(pub(crate) Packet32);

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
        self.data()[0] & 0x0f
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
    pub fn program_change_value(&self) -> u8 {
        self.data()[1]
    }

    /// Pitch bend value data.
    pub fn pitch_bend_value(&self) -> u16 {
        let msb = (self.data()[1] as u16) << 7;
        let lsb = self.data()[2] as u16;
        msb | lsb
    }

    /// Polyphonic key pressure note
    pub fn poly_pressure_note(&self) -> u8 {
        self.data()[1]
    }

    /// Polyphonic key pressure value data.
    pub fn poly_pressure_value(&self) -> u8 {
        self.data()[2]
    }

    /// Channel key pressure value data.
    pub fn channel_pressure_value(&self) -> u8 {
        self.data()[1]
    }

    /// Builder function for adding a channel.
    pub fn with_channel(mut self, channel: u8) -> Self {
        debug_assert!(channel < 16, "Channels must be in the range [0, 15].");
        // 0x30xk_dddd
        let channel = (channel as u32) << 24;
        self.0[0] = (self.0[0] & 0xfff0_0000) | channel;
        self
    }

    /// Create a new note off message.
    pub fn note_off(note: u8, velocity: u8) -> Self {
        debug_assert!(note < 128, "Note numbers must be in the range [0, 127].");
        debug_assert!(velocity < 128, "Velocity must be in the range [0, 127].");
        Self(Packet([0x2080_0000
            | (note as u32) << 8
            | (velocity as u32)]))
    }

    /// Create a new note off message.
    pub fn note_on(note: u8, velocity: u8) -> Self {
        debug_assert!(note < 128, "Note numbers must be in the range [0, 127].");
        debug_assert!(velocity < 128, "Velocity must be in the range [0, 127].");
        Self(Packet([0x2090_0000
            | (note as u32) << 8
            | (velocity as u32)]))
    }

    /// Create a new polyphonic key pressure message.
    pub fn poly_pressure(note: u8, pressure: u8) -> Self {
        debug_assert!(note < 128, "Note numbers must be in the range [0, 127].");
        debug_assert!(pressure < 128, "Pressure must be in the range [0, 127].");
        Self(Packet([0x20a0_0000
            | (note as u32) << 8
            | (pressure as u32)]))
    }

    /// Create a new control change (CC) message.
    pub fn control_change(index: u8, value: u8) -> Self {
        debug_assert!(
            index < 128,
            "Control indices must be in the range [0, 127]."
        );
        debug_assert!(value < 128, "Control values must be in the range [0, 127].");
        Self(Packet([0x20b0_0000 | (index as u32) << 8 | (value as u32)]))
    }

    /// Create a program change message.
    pub fn program_change(note: u8, velocity: u8) -> Self {
        debug_assert!(note < 128, "Program change must be in the range [0, 127].");
        debug_assert!(velocity < 128, "Velocity must be in the range [0, 127].");
        Self(Packet([0x20c0_0000
            | (note as u32) << 8
            | (velocity as u32)]))
    }

    /// Create a new channel pressure message.
    pub fn channel_pressure(value: u8) -> Self {
        debug_assert!(
            value < 128,
            "Channel pressure must be in the range [0, 127]."
        );
        Self(Packet([0x20d0_0000 | (value as u32) << 8]))
    }

    /// Create a pitch bend message.
    pub fn pitch_bend(value: u16) -> Self {
        debug_assert!(value < (1 << 14), "Pitch bend out of range.");
        let value = u32::from_be_bytes([0, 0, (value >> 7) as u8, ((value << 7) >> 7) as u8]);
        Self(Packet([0x20e0_0000 | value]))
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
            stat => unreachable!(
                "invalid status byte for legacy channel voice message: {:x}",
                stat
            ),
        }
    }

    fn data(&self) -> Self::Data {
        (&self.0[0].to_be_bytes()[1..=3]).try_into().unwrap()
    }
}

#[cfg(not(feature = "no-std"))]
impl std::fmt::Display for LegacyChannelVoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let channel = self.channel();
        match self.status() {
            LegacyChannelVoiceStatus::ChannelPressure => {
                write!(f, "ch {channel} pressure {}", self.channel_pressure_value())
            }
            LegacyChannelVoiceStatus::ControlChange => {
                write!(f, "ch {channel} cc {}:{}", self.cc_index(), self.cc_value())
            }
            LegacyChannelVoiceStatus::NoteOff => write!(
                f,
                "ch {channel} note off {}:{}",
                self.note_number(),
                self.velocity()
            ),
            LegacyChannelVoiceStatus::NoteOn => write!(
                f,
                "ch {channel} note on {}:{}",
                self.note_number(),
                self.velocity()
            ),
            LegacyChannelVoiceStatus::PitchBend => {
                write!(f, "ch {channel} pitch bnd {}", self.pitch_bend_value())
            }
            LegacyChannelVoiceStatus::PolyPressure => write!(
                f,
                "ch {channel} poly pressure {}:{}",
                self.poly_pressure_note(),
                self.poly_pressure_value()
            ),
            LegacyChannelVoiceStatus::ProgramChange => {
                write!(f, "ch {channel} prog chng {}", self.program_change_value())
            }
        }
    }
}

impl From<LegacyChannelVoice> for ChannelVoice {
    fn from(value: LegacyChannelVoice) -> Self {
        #[inline(always)]
        fn convert_velocity(v: u8) -> u16 {
            (v as u16) << 8
        }

        #[inline(always)]
        fn convert_pressure(v: u8) -> u32 {
            (v as u32) << 24
        }

        #[inline(always)]
        fn convert_pitch_bend(v: u16) -> u32 {
            (v as u32) << 14
        }

        let message = match value.status() {
            LegacyChannelVoiceStatus::NoteOff => ChannelVoice::note_off(
                value.note_number(),
                convert_velocity(value.velocity()),
                None,
            ),
            LegacyChannelVoiceStatus::NoteOn => ChannelVoice::note_on(
                value.note_number(),
                convert_velocity(value.velocity()),
                None,
            ),
            LegacyChannelVoiceStatus::PolyPressure => ChannelVoice::poly_pressure(
                value.poly_pressure_note(),
                convert_pressure(value.poly_pressure_value()),
            ),
            LegacyChannelVoiceStatus::ControlChange => {
                ChannelVoice::control_change(value.cc_index(), convert_pressure(value.cc_value()))
            }
            LegacyChannelVoiceStatus::ChannelPressure => {
                ChannelVoice::channel_pressure(convert_pressure(value.channel_pressure_value()))
            }
            LegacyChannelVoiceStatus::PitchBend => {
                ChannelVoice::pitch_bend(convert_pitch_bend(value.pitch_bend_value()))
            }
            LegacyChannelVoiceStatus::ProgramChange => {
                ChannelVoice::program_change(0, value.program_change_value(), 0)
            }
        };
        message.with_channel(value.channel())
    }
}
