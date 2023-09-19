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

    pub fn pitch_bend(&self) -> u32 {
        self.data().2
    }

    pub fn poly_pressure(&self) -> u32 {
        self.data().2
    }

    pub fn rpn_bank(&self) -> u8 {
        self.data().0
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

    pub fn program_change(&self) -> ProgramChange {
        let data_bytes = self.data().2.to_ne_bytes();

        // Program Change message contains a single `Bank Valid Bit` flag, which determines if
        // the receiver shall select bank + program, or the program only.
        // The state of this option is represented as `Option` type.
        let should_select_bank: bool = self.data().1 & 0b0000_0001 == 1;

        ProgramChange {
            bank: if should_select_bank {
                Some(u16::from_ne_bytes([data_bytes[1], data_bytes[2]]))
            } else {
                None
            },
            program: data_bytes[0],
            option_flags: self.data().1 >> 1,
        }
    }
}

pub struct ProgramChange {
    pub bank: Option<u16>,
    pub program: u8,
    // Reserved
    option_flags: u8,
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum NoteAttribute {
    /// Receiver shall ignore Attribute Value
    None,
    /// Receiver shall interpret Attribute Value as determined by manufacturer
    ManufacturerSpecific(u16),
    /// Receiver shall interpret Attribute Value as determined by current MIDI-CI profile
    ProfileSpecific(u16),
    ///
    Pitch7_9(u16),
    Reserved,
}

/// RPN Indices defined in MIDI specification.
/// Only pertains to RPN Bank 0.
pub enum RegisteredCtlStatus {
    /// 0x0000. Sets pitch bend range in HCUs and cents
    PitchBendRange { semitones: u8, cents: u8 },
    /// 0x0002. Set the tuning
    CoarseTuning(u8),
    /// 0x0003. Selects a tuning program
    TuningProgramChange(u8),
    /// 0x0004. Selects a tuning bank
    TuningBankSelect(u8),
    /// 0x0006. Declares the number of Channels used for an MPE Lower or Upper Zone
    MPEConfiguration(u8),
    /// 0x0007. Sets per-note pitch bend range in HCUs and cents
    PerNotePitchBendRange(u32),
    Other(u32),
}

pub enum RegisteredPerNoteCtlStatus {
    Modulation(u32),
    Breath(u32),
    Pitch7_25(u32),
    Volume(u32),
    Balance(u32),
    Pan(u32),
    Expression(u32),
    /// defaults to Sound Variation
    SoundController1(u32),
    /// defaults to Timbre/Harmonic intensity
    SoundController2(u32),
    /// defaults to Release Time
    SoundController3(u32),
    /// defaults to Attack Time
    SoundController4(u32),
    /// defaults to Brightness
    SoundController5(u32),
    /// defaults to Decay Time
    SoundController6(u32),
    /// defaults to Vibrato Rate
    SoundController7(u32),
    /// defaults to Vibrato Depth
    SoundController8(u32),
    /// defaults to Vibrato Delay
    SoundController9(u32),
    SoundController10(u32),
    /// defaults to Reverb Send Level
    FX1Depth(u32),
    /// formerly Tremolo depth
    FX2Depth(u32),
    /// defaults to Chorus Send Level
    FX3Depth(u32),
    /// formerly Celeste [Detune] depth
    FX4Depth(u32),
    /// formerly Phaser depth
    FX5Depth(u32),
    Other { bank: u8, index: u8, data: u32 },
}

impl From<(u8, u8, u32)> for RegisteredPerNoteCtlStatus {
    fn from(value: (u8, u8, u32)) -> Self {
        let data = value.2;
        match value.0 {
            1 => Self::Modulation(data),
            2 => Self::Breath(data),
            3 => Self::Pitch7_25(data),
            7 => Self::Volume(data),
            8 => Self::Balance(data),
            10 => Self::Pan(data),
            11 => Self::Expression(data),
            70 => Self::SoundController1(data),
            71 => Self::SoundController2(data),
            72 => Self::SoundController3(data),
            73 => Self::SoundController4(data),
            74 => Self::SoundController5(data),
            75 => Self::SoundController6(data),
            76 => Self::SoundController7(data),
            77 => Self::SoundController8(data),
            78 => Self::SoundController9(data),
            79 => Self::SoundController10(data),
            91 => Self::FX1Depth(data),
            92 => Self::FX2Depth(data),
            93 => Self::FX3Depth(data),
            94 => Self::FX4Depth(data),
            95 => Self::FX5Depth(data),
            _ => Self::Other {
                bank: value.0,
                index: value.1,
                data,
            },
        }
    }
}
