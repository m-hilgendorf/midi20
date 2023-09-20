//! MIDI 2.0 channel voice messages.
use core::ops::Deref;

use crate::message::Message;
use crate::packet::{MessageType, Packet64};

/// MIDI 2.0 channel voice messages
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct ChannelVoice(Packet64);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum ChannelVoiceStatus {
    NoteOff = 0x8,
    NoteOn = 0x9,
    PolyPressure = 0xa,
    RpnMgmt = 0x0,
    ArpnMgmt = 0x1,
    PerNoteMgmt = 0xf,
    ControlChange = 0xb,
    RpnControlChange = 0x2,
    ArpnControlChange = 0x3,
    RpnRelativeControlChange = 0x4,
    ArpnRelativeControlChange = 0x5,
    ProgramChange = 0xc,
    ChannelPressure = 0xd,
    PitchBend = 0xe,
    PerNotePitchBend = 0x6,
}
impl ChannelVoice {
    pub(crate) fn from_packet_unchecked(ump: Packet64) -> Self {
        Self(ump)
    }

    /// The note number data, for relevent message statuses.
    pub fn note_number(&self) -> u8 {
        self.data().0
    }

    /// The velocity data, for relevant statuses.
    pub fn velocity(&self) -> u16 {
        (self.data().2 >> 16) as u16
    }

    /// Note On/Off attribute data.
    // TODO: more specific type.
    // 0 => None
    // 1 => Manufacturer Specific
    // 2 => Profile Specific
    // 3 => Pitch 7.9
    pub fn attribute_data(&self) -> u16 {
        (self.data().2 & 0x0000_ffff) as u16
    }

    /// Polyphonic key pressure value data.
    pub fn poly_pressure(&self) -> u32 {
        self.data().2
    }

    /// Registered per-note control index data.
    pub fn rpn_index(&self) -> u8 {
        self.data().1
    }

    /// Registered per-note control value data.
    pub fn rpn_data(&self) -> u32 {
        self.data().2
    }

    /// Per note management flags.
    pub fn per_note_mgmt_flags(&self) -> u8 {
        self.data().1
    }

    /// Control change index data.
    pub fn cc_index(&self) -> u8 {
        self.data().0
    }

    /// Control change value data.
    pub fn cc_value(&self) -> u32 {
        self.data().2
    }

    /// Program change value data.
    // TODO: more specific type.
    pub fn program_change(&self) -> u8 {
        todo!()
    }

    /// Pitch bend value data.
    pub fn pitch_bend(&self) -> u32 {
        self.data().2
    }
}

impl Deref for ChannelVoice {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for ChannelVoice {
    type Status = ChannelVoiceStatus;
    type Data = (u8, u8, u32);

    fn message_type(&self) -> MessageType {
        let type_ = self.0.message_type().into();
        debug_assert!(type_ == MessageType::ChannelVoice, "Invalid message type..");
        type_
    }

    fn group(&self) -> u8 {
        self.0.group()
    }

    fn status(&self) -> Self::Status {
        match (self.0[0] >> 20) & 0xf {
            0x8 => Self::Status::NoteOff,
            0x9 => Self::Status::NoteOn,
            0xa => Self::Status::PolyPressure,
            0x0 => Self::Status::RpnMgmt,
            0x1 => Self::Status::ArpnMgmt,
            0xf => Self::Status::PerNoteMgmt,
            0xb => Self::Status::ControlChange,
            0x2 => Self::Status::RpnControlChange,
            0x3 => Self::Status::ArpnControlChange,
            0x4 => Self::Status::RpnRelativeControlChange,
            0x5 => Self::Status::ArpnRelativeControlChange,
            0xc => Self::Status::ProgramChange,
            0xd => Self::Status::ChannelPressure,
            0xe => Self::Status::PitchBend,
            0x6 => Self::Status::PerNotePitchBend,
            _ => unreachable!("Invalid status byte for channel voice message."),
        }
    }

    fn data(&self) -> Self::Data {
        let word1 = self.0[0].to_ne_bytes();
        let word2 = self.0[1];
        (word1[2], word1[3], word2)
    }
}
