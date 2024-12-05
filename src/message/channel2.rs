//! MIDI 2.0 channel voice messages.
use core::ops::Deref;

use crate::message::Message;
use crate::packet::{MessageType, Packet, Packet64};

/// MIDI 2.0 channel voice messages
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct ChannelVoice(pub(crate) Packet64);

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

/// Attributes that can be set on NoteON or NoteOFF messages
#[derive(Copy, Clone, Debug)]
pub enum Attribute {
    /// A manufacturer specified attribute.
    Manufacturer(u16),

    /// A profile specified attribute.
    Profile(u16),

    /// Pitch control, in 7.9 fixed point.
    Pitch79(u16),
}

impl ChannelVoice {
    pub(crate) fn from_packet_unchecked(ump: Packet64) -> Self {
        Self(ump)
    }

    /// The destination channel for this message.
    pub fn channel(&self) -> u8 {
        ((self.0[0] >> 16) & 0xf) as u8
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
    pub fn poly_pressure_value(&self) -> u32 {
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

    /// Registered per-note control index data.
    pub fn arpn_index(&self) -> u8 {
        self.data().1
    }

    /// Registered per-note control value data.
    pub fn arpn_data(&self) -> u32 {
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
    pub fn program_change_value(&self) -> u8 {
        todo!()
    }

    /// Pitch bend value data.
    pub fn pitch_bend_value(&self) -> u32 {
        self.data().2
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
    pub fn note_off(note: u8, velocity: u16, attribute: Option<Attribute>) -> Self {
        debug_assert!(note < 128, "Note numbers must be in the range [0, 127].");
        let (attr_type, attr_data) = match attribute {
            None => (0, 0),
            Some(Attribute::Manufacturer(data)) => (1, data as u32),
            Some(Attribute::Profile(data)) => (2, data as u32),
            Some(Attribute::Pitch79(data)) => (3, data as u32),
        };
        let velocity = (velocity as u32) << 16;
        let note_number = (note as u32) << 8;
        Self(Packet([
            0x4090_0000 | note_number | attr_type,
            velocity | attr_data,
        ]))
    }

    /// Create a new note off message.
    pub fn note_on(note: u8, velocity: u16, attribute: Option<Attribute>) -> Self {
        debug_assert!(note < 128, "Note numbers must be in the range [0, 127].");
        let (attr_type, attr_data) = match attribute {
            None => (0, 0),
            Some(Attribute::Manufacturer(data)) => (1, data as u32),
            Some(Attribute::Profile(data)) => (2, data as u32),
            Some(Attribute::Pitch79(data)) => (3, data as u32),
        };
        let velocity = (velocity as u32) << 16;
        let note_number = (note as u32) << 8;
        Self(Packet([
            0x40a0_0000 | note_number | attr_type,
            velocity | attr_data,
        ]))
    }

    /// Create a new polyphonic key pressure message.
    pub fn poly_pressure(note: u8, pressure: u32) -> Self {
        debug_assert!(note < 128, "Note numbers must be in the range [0, 127].");
        let note_number = (note as u32) << 8;
        Self(Packet([0x40b0_0000 | note_number, pressure]))
    }

    /// Create a new control change (CC) message.
    pub fn control_change(index: u8, value: u32) -> Self {
        debug_assert!(
            index < 128,
            "Control indices must be in the range [0, 127]."
        );
        debug_assert!(value < 128, "Control values must be in the range [0, 127].");
        Self(Packet([0x20b0_0000 | (index as u32) << 8, value]))
    }

    /// Create a program change message.
    pub fn program_change(options: u8, program: u8, bank: u16) -> Self {
        let (bank_msb, bank_lsb) = {
            let bytes = bank.to_be_bytes();
            (bytes[0], bytes[1])
        };
        Self(Packet([
            0x20c0_0000 | (options as u32),
            (program as u32) << 24 | (bank_msb as u32) << 8 | (bank_lsb as u32),
        ]))
    }

    /// Create a new channel pressure message.
    pub fn channel_pressure(value: u32) -> Self {
        Self(Packet([0x20d0_0000, value]))
    }

    /// Create a pitch bend message.
    pub fn pitch_bend(value: u32) -> Self {
        let value = u32::from_be_bytes([0, 0, (value >> 7) as u8, ((value << 7) >> 7) as u8]);
        Self(Packet([0x20e0_0000, value]))
    }

    /// Create a per-note pitch bend message.
    pub fn per_note_pitch_bend(note_number: u8, value: u32) -> Self {
        let value = u32::from_be_bytes([0, 0, (value >> 7) as u8, ((value << 7) >> 7) as u8]);
        Self(Packet([0x20e0_0000 | (note_number as u32) << 8, value]))
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
