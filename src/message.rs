//! MIDI 2.0 Message Types
//!
//! MIDI 2.0 Messages form a mostly flat abstract syntax tree. MIDI 1.0 types are represented by
//! the [LegacyChannelVoice] enum.
use crate::packet::*;
use core::convert::TryInto;
use core::ops::Deref;

pub trait Message
where
    Self: Deref<Target = [u32]>,
{
    type Data;
    type Status;

    fn message_type(&self) -> MessageType;
    fn group(&self) -> u8;
    fn status(&self) -> Self::Status;
    fn data(&self) -> Self::Data;
}

/// Parent type of each MIDI Message
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum MidiMessageData {
    /// Utility types (Jitter reduction, No-op)
    Utility(Utility),

    /// System common types (midi time code, song position, tune request, song number)
    System(System),

    /// System real time types (start, stop, continue, timing clock, active sensing, reset)
    SystemRealtime(SystemRealtime),

    /// MIDI 1.0 channel voice messages (note on/off, keypressure, program change, control change)
    LegacyChannelVoice(LegacyChannelVoice),

    /// MIDI 2.0 channel voice messages
    ChannelVoice(ChannelVoice),

    /// 64 bit data messages
    Data64(Data64),

    /// 128 bit data messages
    Data128(Data128),

    Reserved32(Packet32),
    Reserved64(Packet64),
    Reserved96(Packet96),
    Reservd128(Packet128),
}

/// Utility messages defined by the MIDI 2.0 specification, including
/// jitter reduction and no-op.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Utility(Packet32);

/// Represents the status byte of a [Utility] message.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum UtilityStatus {
    NoOp = 0,
    Jr = 1,
    JrClock = 2,
}

impl Utility {
    pub(crate) fn from_packet_unchecked(ump: Packet32) -> Self {
        Self(ump)
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
            1 => Self::Status::Jr,
            2 => Self::Status::JrClock,
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

impl System {
    pub(crate) fn from_packet_unchecked(ump: Packet32) -> Self {
        Self(ump)
    }

    pub fn time_code(&self) -> u8 {
        self.data()[0]
    }

    pub fn song_select(&self) -> u8 {
        self.data()[0]
    }

    pub fn song_position_pointer(&self) -> u16 {
        u16::from_be_bytes(self.data())
    }
}

/// System realtime messages, for transport, timing clock, active sensing, and reset
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct SystemRealtime(Packet32);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SystemRealtimeStatus {
    TimingClock = 0xf8,
    Start = 0xfa,
    Continue = 0xfb,
    Stop = 0xfc,
    ActiveSensing = 0xfe,
    Reset = 0xff,
}

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

/// MIDI 2.0 channel voice messages
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct ChannelVoice(Packet64);

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
#[repr(u8)]
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
    pub fn attribute_data(&self) -> u16 {
        (self.data().2 & 0x0000_ffff) as u16
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
pub enum DataStatus {
    SinglePacket = 0x0,
    Start = 0x1,
    Continue = 0x2,
    End = 0x3,
    MixedDataSetHeader = 0x4,
    MixedDataSetPayload = 0x5,
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Data64(Packet64);

impl Data64 {
    pub(crate) fn from_packet_unchecked(ump: Packet64) -> Self {
        Self(ump)
    }
}

impl Deref for Data64 {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for Data64 {
    type Status = DataStatus;
    type Data = [u8; 6];

    fn message_type(&self) -> MessageType {
        let type_ = self.0.message_type().into();
        debug_assert!(type_ == MessageType::Data64, "Invalid message type..");
        type_
    }

    fn group(&self) -> u8 {
        self.0.group()
    }

    fn status(&self) -> Self::Status {
        match (self.0[0] >> 20) & 0xf {
            0x0 => Self::Status::SinglePacket,
            0x1 => Self::Status::Start,
            0x2 => Self::Status::Continue,
            0x3 => Self::Status::End,
            _ => unreachable!("Invalid status byte for 8 byte data message."),
        }
    }

    fn data(&self) -> Self::Data {
        todo!()
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Data128(Packet128);

impl Deref for Data128 {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Data128 {
    pub(crate) fn from_packet_unchecked(ump: Packet128) -> Self {
        Self(ump)
    }
}

impl Message for Data128 {
    type Status = DataStatus;
    type Data = [u8; 12];

    fn message_type(&self) -> MessageType {
        let type_ = self.0.message_type().into();
        debug_assert!(type_ == MessageType::Data128, "Invalid message type..");
        type_
    }

    fn group(&self) -> u8 {
        self.0.group()
    }

    fn status(&self) -> Self::Status {
        match (self.0[0] >> 20) & 0xf {
            0x0 => Self::Status::SinglePacket,
            0x1 => Self::Status::Start,
            0x2 => Self::Status::Continue,
            0x3 => Self::Status::End,
            0x4 => Self::Status::MixedDataSetHeader,
            0x5 => Self::Status::MixedDataSetPayload,
            _ => unreachable!("Invalid status byte for 16 byte data message."),
        }
    }

    fn data(&self) -> Self::Data {
        todo!()
    }
}
