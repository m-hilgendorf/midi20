use core::ops::Deref;

use crate::packet::{MessageType, Packet64};

use super::Message;

/// MIDI 2.0 channel voice messages
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct ChannelVoice(Packet64);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ChannelVoiceStatus {
    NoteOff = 0x8,
    NoteOn = 0x9,
    PolyPressure = 0xa,
    ControlChange = 0xb,
    ProgramChange = 0xc,
    ChannelPressure = 0xd,
    PitchBend = 0xe,

    RegisteredPerNoteCtl = 0x0,
    AssignablePerNoteCtl = 0x1,
    RegisteredCtl = 0x2,
    AssignableCtl = 0x3,
    RelRegisteredPerNoteCtl = 0x4,
    RelAssignablePerNoteCtl = 0x5,
    PerNotePitchBend = 0x6,
    PerNoteManagement = 0xf,
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
            0xb => Self::Status::ControlChange,
            0xc => Self::Status::ProgramChange,
            0xd => Self::Status::ChannelPressure,
            0xe => Self::Status::PitchBend,

            0x0 => Self::Status::RegisteredPerNoteCtl,
            0x1 => Self::Status::AssignablePerNoteCtl,
            0x2 => Self::Status::RegisteredCtl,
            0x3 => Self::Status::AssignableCtl,
            0x4 => Self::Status::RelRegisteredPerNoteCtl,
            0x5 => Self::Status::RelAssignablePerNoteCtl,
            0x6 => Self::Status::PerNotePitchBend,
            0xF => Self::Status::PerNoteManagement,
            _ => unreachable!("Invalid status byte for channel voice message."),
        }
    }

    fn data(&self) -> Self::Data {
        let word1 = self.0[0].to_ne_bytes();
        let word2 = self.0[1];
        (word1[2], word1[3], word2)
    }
}

impl ChannelVoice {
    pub(crate) fn from_packet_unchecked(ump: Packet64) -> Self {
        Self(ump)
    }

    pub fn note_number(&self) -> u8 {
        self.data().0
    }

    pub fn velocity(&self) -> u16 {
        (self.data().2 >> 16) as u16
    }

    // TODO: more specific type.
    // 0 => None
    // 1 => Manufacturer Specific
    // 2 => Profile Specific
    // 3 => Pitch 7.9
    pub fn attribute_data(&self) -> Result<NoteAttribute, ()> {
        let attribute_type = self.data().1;
        let get_attribute_value = || (self.data().2 & 0x0000_FFFF) as u16;
        match attribute_type {
            0x00 => Ok(NoteAttribute::None),
            0x01 => Ok(NoteAttribute::ManufacturerSpecific(get_attribute_value())),
            0x02 => Ok(NoteAttribute::ProfileSpecific(get_attribute_value())),
            0x03 => Ok(NoteAttribute::Pitch7_9(get_attribute_value())),
            _ => Err(()),
        }
    }

    pub fn poly_pressure(&self) -> u32 {
        self.data().2
    }

    pub fn rpn_index(&self) -> u8 {
        self.data().1
    }

    pub fn rpn_data(&self) -> u32 {
        self.data().2
    }

    pub fn per_note_mgmt_flags(&self) -> u8 {
        self.data().1
    }

    pub fn cc_index(&self) -> u8 {
        self.data().0
    }

    pub fn cc_value(&self) -> u32 {
        self.data().2
    }

    // TODO: more specific type.
    pub fn program_change(&self) -> u8 {
        todo!()
    }

    pub fn pitch_bend(&self) -> u32 {
        self.data().2
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum NoteAttribute {
    None,
    ManufacturerSpecific(u16),
    ProfileSpecific(u16),
    Pitch7_9(u16),
}

impl NoteAttribute {}