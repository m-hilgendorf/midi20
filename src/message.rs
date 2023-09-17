//! MIDI 2.0 Message Types
//!
//! MIDI 2.0 Messages form a mostly flat abstract syntax tree. MIDI 1.0 types are represented by
//! the [LegacyChannelVoice] enum.
use crate::ump::UMP;

/// Trait used for easy conversion from lower types into to the message type
pub trait IntoMidiMessage {
    /// Convert into a complete message by assigning a group
    fn into_msg(self, group: u8) -> MidiMessage;
}

/// A MIDI message, parsed from a Universal Midi Packet
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct MidiMessage {
    /// The group this message is destined for, must be less than 16.
    pub group: u8,
    /// The underlying MIDI data
    pub data: MidiMessageData,
}

impl MidiMessage {
    /// Returns the group of the type. Panics in debug mode if the group is invalid.
    pub fn group(&self) -> u8 {
        debug_assert!(self.group < 16);
        self.group & 0x0f
    }
}

/// Parent type of each MIDI Message
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum MidiMessageData {
    /// Utility types (Jitter reduction, No-op)
    Utility(Utility),
    /// System common types (midi time code, song position, tune request, song number)
    SystemCommon(SystemCommon),
    /// System real time types (start, stop, continue, timing clock, active sensing, reset)
    SystemRealtime(SystemRealtime),
    /// MIDI 1.0 channel voice messages (note on/off, keypressure, program change, control change)
    LegacyChannelVoice(LegacyChannelVoice),
    /// MIDI 2.0 channel voice messages
    ChannelVoice(ChannelVoice),
    /// Flex data messages
    Flex(Flex),
    /// 64 bit data messages
    Data64(Data64),
    /// 128 bit data messages
    Data128(Data128),
    /// Undefined messages at the time of writing, including all 96 bit messages
    Undefined(UMP),
}

impl IntoMidiMessage for MidiMessageData {
    fn into_msg(self, group: u8) -> MidiMessage {
        debug_assert!(group < 16);
        MidiMessage { group, data: self }
    }
}
/// Utility messages defined by the MIDI 2.0 specification, including
/// jitter reduction and no-op.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum Utility {
    /// No operation
    NoOp,
    /// A jitter reduction synchronization message.
    JrClock(u16),
    /// A jitter reduction timestamp sent by a receiver.
    JrTimestamp(u16),
}

impl IntoMidiMessage for Utility {
    fn into_msg(self, group: u8) -> MidiMessage {
        debug_assert!(group < 16);
        MidiMessage {
            group,
            data: MidiMessageData::Utility(self),
        }
    }
}

/// System common messages, for time code, song position, song select, and tune request
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum SystemCommon {
    /// A new piece of MIDI time code for
    TimeCode(u8),
    /// Updates the current position of a song.
    SongPositionPointer(u16),
    /// Updates which song is playing
    SongSelect(u8),
    /// Request to re-tune the device
    TuneRequest,
}

impl IntoMidiMessage for SystemCommon {
    fn into_msg(self, group: u8) -> MidiMessage {
        debug_assert!(group < 16);
        MidiMessage {
            group,
            data: MidiMessageData::SystemCommon(self),
        }
    }
}

/// System realtime messages, for transport, timing clock, active sensing, and reset
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum SystemRealtime {
    /// Clock synchronization
    TimingClock,
    /// Transport: begin playing
    Start,
    /// Transport: continue playing
    Continue,
    /// Transport: stop playing
    Stop,
    /// Active sensing message
    ActiveSensing,
    /// Reset message
    Reset,
}

impl IntoMidiMessage for SystemRealtime {
    fn into_msg(self, group: u8) -> MidiMessage {
        debug_assert!(group < 16);
        MidiMessage {
            group,
            data: MidiMessageData::SystemRealtime(self),
        }
    }
}

/// MIDI 1.0 Channel Voice Messages
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum LegacyChannelVoice {
    /// A note ON message (begin playing)
    NoteOn { channel: u8, note: u8, vel: u8 },
    /// A note OFF message (stop playing)
    NoteOff { channel: u8, note: u8, vel: u8 },
    /// Polyphonic key pressure, also called aftertouch.
    PolyPressure { channel: u8, note: u8, pressure: u8 },
    /// A control change (CC) message.
    ControlChange { channel: u8, control: u8, value: u8 },
    /// A program change message.
    ProgramChange {
        channel: u8,
        program: u8,
        _reserved: u8,
    },
    /// Channel pressure, also called aftertouch.
    ChannelPressure {
        channel: u8,
        pressure: u8,
        _reserved: u8,
    },
    /// A pitch bend message. Data is a 14 bit unsigned value.
    PitchBend { channel: u8, data: u16 },
}

impl IntoMidiMessage for LegacyChannelVoice {
    fn into_msg(self, group: u8) -> MidiMessage {
        debug_assert!(group < 16);
        MidiMessage {
            group,
            data: MidiMessageData::LegacyChannelVoice(self),
        }
    }
}

/// MIDI 2.0 channel voice messages
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum ChannelVoice {
    /// A registered parameter, per-note. Like a CC, but only applied to a single
    /// voice and pulled from a set of registered parameter numbers (RPN).
    RegPerNoteCtrl {
        channel: u8,
        note: u8,
        control: u8,
        data: u32,
    },
    /// An assignmable parameter. Like a CC, but only applied to a single voice
    /// and may be assignable by the the user or device (ARPN).
    AsgnPerNoteCtrl {
        channel: u8,
        note: u8,
        control: u8,
        data: u32,
    },
    /// A registered control. Like a CC, but with a registered parameter number (RPN).
    RegsteredCtrl {
        channel: u8,
        bank: u8,
        index: u8,
        data: u32,
    },
    /// An assignmable control. Like a CC, but may be assigned by the the user or device (ARPN)
    AssignableCtrl {
        channel: u8,
        bank: u8,
        index: u8,
        data: u32,
    },
    /// A relative registered control.
    RelRegCtrl {
        channel: u8,
        bank: u8,
        index: u8,
        data: u32,
    },
    /// A relative assignable control.
    RelAssnCtrl {
        channel: u8,
        bank: u8,
        index: u8,
        data: u32,
    },
    /// Pitch bend, but only for one voice.
    PerNotePitchBnd { channel: u8, note: u8, data: u32 },
    /// A note OFF message (stop playing).
    NoteOff {
        channel: u8,
        note: u8,
        attr: u8,
        vel: u16,
        attr_val: u16,
    },
    /// A note ON message (begin playing)
    NoteOn {
        channel: u8,
        note: u8,
        attr: u8,
        vel: u16,
        attr_val: u16,
    },
    /// Polyphonic key pressure, also called aftertouch
    PolyPressure { channel: u8, note: u8, data: u32 },
    /// A control change (CC) message
    ControlChange { channel: u8, control: u8, data: u32 },
    /// A program change message
    ProgramChange {
        channel: u8,
        options: u8,
        program: u8,
        bank: u16,
    },
    /// Channel pressure message, also called aftertouch
    ChannelPressure { channel: u8, data: u32 },
    /// Channel-wide pitch bend, applied to every note.
    PitchBend { channel: u8, data: u32 },
    /// A per-note management message.
    PerNoteMngmt { channel: u8, note: u8, flags: u8 },
}

impl IntoMidiMessage for ChannelVoice {
    fn into_msg(self, group: u8) -> MidiMessage {
        debug_assert!(group < 16);
        MidiMessage {
            group,
            data: MidiMessageData::ChannelVoice(self),
        }
    }
}

/// Flex data messages: real time messages with limited variability of size.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum Flex {
    /// Sets musical tempo by declaring the number of 10 nanosecond units per quarter note.
    SetTempo(u32),
    /// Declares and sets a Time Signature for subsequent bars.
    SetTimeSignature {
        /// A value from 1 to 256, supporting up to 256 beats in a bar.
        numerator: u8,
        /// A value in negative power of 2 (2 represents a quarter note, 3 represents an eighth note, etc.) If the value is set to zero, there is a non-standard denominator.
        denominator: u8,
        /// Expresses the number of 1/32 notes in 24 MIDI Clocks.
        number_of_32n: u8,
    },
    /// Sets metronome functions.
    SetMetronome {
        /// Number of MIDI Clocks per Primary Click.
        /// 1 tick is 1/96
        /// 24 ticks is 1/4
        clocks_per_click: u8,
        /// Sets the number of Primary Clicks between each accent.
        bar_accents: [u8; 3],
        /// Sets number of clicks in between Primary Clicks.
        subdivision_clicks: [u8; 2],
    },
    /// Sets the Key Signature for up to 7 sharps or up to 7 flats
    SetKeySignature {
        address: FlexAddress,
        /// The number of sharps(+) or flats(-).
        sharps_or_flats: i8,
        /// The tonic note (A = 1, G = 7, others are non-standard)
        tonic_note: u8,
    },
    /// Declares the name of a chord
    SetChordName(FlexAddress, [u8; 12]),
    /// Contains text encoded in UTF-8 format
    TextMessageCommonFormat {
        /// determines the address destination of each UMP: group, channel, or <reserved>
        address: FlexAddress,
        /// determines the role of each UMP in a Flex Data Message
        format: FlexFormat,
        /// provides up to 256 (text) message classifications
        status: FlexStatus,
        data: [u8; 12],
    },
}

/// Determines the destination of each UMP.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum FlexAddress {
    Channel(u8),
    Group(u8),
    _Reserved1(u8),
    _Reserved2(u8),
}

impl From<FlexAddress> for u8 {
    fn from(value: FlexAddress) -> Self {
        match value {
            FlexAddress::Channel(channel) => 0 << 4 | channel,
            FlexAddress::Group(group) => 1 << 4 | group,
            FlexAddress::_Reserved1(cccc) => 2 << 4 | cccc,
            FlexAddress::_Reserved2(cccc) => 3 << 4 | cccc,
        }
    }
}

impl From<u8> for FlexAddress {
    fn from(value: u8) -> Self {
        let status = value >> 4 & 0b0011;
        let cccc = value & 0x0F;
        match status {
            0 => FlexAddress::Channel(cccc),
            1 => FlexAddress::Group(cccc),
            2 => FlexAddress::_Reserved1(cccc),
            3 => FlexAddress::_Reserved2(cccc),
            _ => unreachable!(),
        }
    }
}

/// Provides up to 256 message classifications for Text Message Common Format.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum FlexStatus {
    SetupAndPerformance, // pertains only to non-Text messages
    MetadataText(FlexMetadataText),
    PerformanceTextEvent(FlexPerformanceTextEvent),
    Reserved { status_bank: u8, status: u8 },
}

impl FlexStatus {
    pub fn from_bytes(status_bank: u8, status: u8) -> Self {
        let joined: u16 = (status_bank as u16) << 8 | status as u16;
        Self::from(joined)
    }

    pub fn status_bank_byte(&self) -> u8 {
        match self {
            FlexStatus::SetupAndPerformance => 0x00,
            FlexStatus::MetadataText(_) => 0x01,
            FlexStatus::PerformanceTextEvent(_) => 0x02,
            FlexStatus::Reserved {
                status_bank,
                status,
            } => *status_bank,
        }
    }

    pub fn status_byte(&self) -> u8 {
        match self {
            FlexStatus::SetupAndPerformance => 0x00,
            FlexStatus::MetadataText(status) => u8::from(*status),
            FlexStatus::PerformanceTextEvent(status) => u8::from(*status),
            FlexStatus::Reserved {
                status_bank,
                status,
            } => *status,
        }
    }
}

impl From<FlexStatus> for u16 {
    fn from(value: FlexStatus) -> Self {
        match value {
            FlexStatus::SetupAndPerformance => 0,
            FlexStatus::MetadataText(status) => 0x01 << 8 | u8::from(status) as u16,
            FlexStatus::PerformanceTextEvent(status) => 0x02 << 8 | u8::from(status) as u16,
            FlexStatus::Reserved {
                status_bank,
                status,
            } => (status_bank as u16) << 8 | status as u16,
        }
    }
}

impl From<u16> for FlexStatus {
    fn from(value: u16) -> Self {
        let status_bank = (value >> 8) as u8;
        let status = (value & 0x00FF) as u8;
        match status_bank {
            0x00 => Self::SetupAndPerformance,
            0x01 => Self::MetadataText(status.into()),
            0x02 => Self::PerformanceTextEvent(status.into()),
            _ => Self::Reserved {
                status_bank,
                status,
            },
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum FlexMetadataText {
    Unknown,
    ProjectName,
    CompositionName,
    MidiClipName,
    CopyrightNotice,
    ComposerName,
    LyricistName,
    ArrangerName,
    PublisherName,
    PrimaryPerformerName,
    AccompanyingPerformerName,
    RecordingDate,
    RecordingLocation,
}

impl From<u8> for FlexMetadataText {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::ProjectName,
            2 => Self::CompositionName,
            3 => Self::MidiClipName,
            4 => Self::CopyrightNotice,
            5 => Self::ComposerName,
            6 => Self::LyricistName,
            7 => Self::ArrangerName,
            8 => Self::PublisherName,
            9 => Self::PrimaryPerformerName,
            10 => Self::AccompanyingPerformerName,
            11 => Self::RecordingDate,
            12 => Self::RecordingLocation,
            _ => unreachable!(),
        }
    }
}

impl From<FlexMetadataText> for u8 {
    fn from(value: FlexMetadataText) -> Self {
        match value {
            FlexMetadataText::Unknown => 0,
            FlexMetadataText::ProjectName => 1,
            FlexMetadataText::CompositionName => 2,
            FlexMetadataText::MidiClipName => 3,
            FlexMetadataText::CopyrightNotice => 4,
            FlexMetadataText::ComposerName => 5,
            FlexMetadataText::LyricistName => 6,
            FlexMetadataText::ArrangerName => 7,
            FlexMetadataText::PublisherName => 8,
            FlexMetadataText::PrimaryPerformerName => 9,
            FlexMetadataText::AccompanyingPerformerName => 10,
            FlexMetadataText::RecordingDate => 11,
            FlexMetadataText::RecordingLocation => 12,
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum FlexPerformanceTextEvent {
    Unknown,
    Lyrics,
    LyricsLanguage,
    Ruby,
    RubyLanguage,
}

impl From<u8> for FlexPerformanceTextEvent {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Lyrics,
            2 => Self::LyricsLanguage,
            3 => Self::Ruby,
            4 => Self::RubyLanguage,
            _ => unreachable!(),
        }
    }
}

impl From<FlexPerformanceTextEvent> for u8 {
    fn from(value: FlexPerformanceTextEvent) -> Self {
        match value {
            FlexPerformanceTextEvent::Unknown => 0,
            FlexPerformanceTextEvent::Lyrics => 1,
            FlexPerformanceTextEvent::LyricsLanguage => 2,
            FlexPerformanceTextEvent::Ruby => 3,
            FlexPerformanceTextEvent::RubyLanguage => 4,
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum FlexFormat {
    SinglePacket,
    Start,
    Continue,
    End,
}

impl FlexFormat {
    pub fn extract_from_byte(byte: u8) -> FlexFormat {
        (byte >> 6).into()
    }
}

impl From<FlexFormat> for u8 {
    fn from(value: FlexFormat) -> Self {
        match value {
            FlexFormat::SinglePacket => 0,
            FlexFormat::Start => 1,
            FlexFormat::Continue => 2,
            FlexFormat::End => 3,
        }
    }
}

impl From<u8> for FlexFormat {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::SinglePacket,
            1 => Self::Start,
            2 => Self::Continue,
            3 => Self::End,
            _ => unreachable!(),
        }
    }
}

impl IntoMidiMessage for Flex {
    fn into_msg(self, group: u8) -> MidiMessage {
        debug_assert!(group < 16);
        MidiMessage {
            group,
            data: MidiMessageData::Flex(self),
        }
    }
}

/// A 64 bit data packet, eg for SysEx.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum Data64 {
    /// All the data is contained in a single packet
    SinglePacket([u8; 8]),
    /// The first packet in a stream of data
    Start([u8; 8]),
    /// A packet in the middle of a stream of data
    Continue([u8; 8]),
    /// The last packet in a stream of data.
    End([u8; 8]),
}

impl Data64 {
    /// get the bytes of the packet
    pub fn bytes(&self) -> [u8; 8] {
        match self {
            Data64::SinglePacket(b) => *b,
            Data64::Start(b) => *b,
            Data64::Continue(b) => *b,
            Data64::End(b) => *b,
        }
    }
}

impl IntoMidiMessage for Data64 {
    fn into_msg(self, group: u8) -> MidiMessage {
        debug_assert!(group < 16);
        MidiMessage {
            group,
            data: MidiMessageData::Data64(self),
        }
    }
}

/// A 128 bit data message, eg for SysEx.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum Data128 {
    /// All data is contained in a single packet
    SinglePacket([u8; 16]),
    /// The first packet in a stream of data
    Start([u8; 16]),
    /// A packet in the middle of a stream of data
    Continue([u8; 16]),
    /// The last packet of a stream of data
    End([u8; 16]),
    /// The header used by a mixed data set packet
    MixedDataSetHeader([u8; 16]),
    /// The payload used by a mixed data set packet
    MixedDataSetPayload([u8; 16]),
}

impl IntoMidiMessage for Data128 {
    fn into_msg(self, group: u8) -> MidiMessage {
        debug_assert!(group < 16);
        MidiMessage {
            group,
            data: MidiMessageData::Data128(self),
        }
    }
}

impl Data128 {
    /// Get the bytes of the message.
    pub fn bytes(&self) -> [u8; 16] {
        match self {
            Data128::SinglePacket(b) => *b,
            Data128::Start(b) => *b,
            Data128::Continue(b) => *b,
            Data128::End(b) => *b,
            Data128::MixedDataSetHeader(b) => *b,
            Data128::MixedDataSetPayload(b) => *b,
        }
    }
}
