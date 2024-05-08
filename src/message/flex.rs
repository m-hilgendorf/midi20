#![allow(missing_docs)]
use core::convert::TryInto;
use core::ops::Deref;

use crate::message::{data::DataFormat, Message};
use crate::packet::{MessageType, Packet128};

/// Flex data messages: real time messages with limited variability of size.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct Flex(pub(crate) Packet128);

impl Deref for Flex {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Message for Flex {
    type Data = [u32; 3];
    type Status = FlexStatus;

    fn message_type(&self) -> MessageType {
        let msg_type = self.0.message_type().into();
        debug_assert!(msg_type == MessageType::Flex, "Invalid message type");
        msg_type
    }

    fn group(&self) -> u8 {
        self.0.group()
    }

    fn status(&self) -> FlexStatus {
        let dword = self.0[0];
        let word = (dword & 0x0000_FFFF) as u16;
        word.into()
    }

    fn data(&self) -> Self::Data {
        self.0[1..4].try_into().unwrap()
    }
}

impl Flex {
    /// Determines the role of each UMP within a Flex Data Message.
    pub fn format(&self) -> DataFormat {
        let dword = self.0[0];
        let byte = dword.to_ne_bytes()[1];
        (byte >> 6).into()
    }

    pub fn address(&self) -> FlexAddress {
        let dword = self.0[0];
        let byte = dword.to_ne_bytes()[1];
        byte.into()
    }

    /// Sets musical tempo by declaring the number of 10 nanosecond units per quarter note.
    pub fn tempo(&self) -> u32 {
        self.data()[0]
    }

    pub fn time_signature(&self) -> FlexTimeSignature {
        let dword = self.data()[0];
        dword.into()
    }

    pub fn metronome(&self) -> FlexMetronome {
        let dwords = self.data();
        [dwords[0], dwords[1]].into()
    }

    pub fn key_signature(&self) -> FlexKeySignature {
        let dword = self.data()[0];
        let byte = dword.to_ne_bytes()[0];
        FlexKeySignature(byte)
    }

    pub fn chord_name(&self) -> FlexChordName {
        self.data().into()
    }

    pub fn text_message(&self) -> &'_ [u8] {
        unsafe {
            let data = self.data().as_ptr().cast();
            core::slice::from_raw_parts(data, 12)
        }
    }

    pub(crate) fn from_packet_unchecked(ump: Packet128) -> Self {
        Self(ump)
    }
}

/// Determines the address destination of each UMP.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum FlexAddress {
    /// Message is sent to the channel.
    Channel(u8),

    /// Message is sent to the group on channel 1.
    Group(u8),

    /// Reserved.
    Reserved1(u8),

    /// Reserved.
    Reserved2(u8),
}

impl From<u8> for FlexAddress {
    fn from(value: u8) -> Self {
        let address = (value >> 4) & 3;
        let destination = value & 0x0F;
        match address {
            0 => Self::Channel(destination),
            1 => Self::Group(destination),
            2 => Self::Reserved1(destination),
            3 => Self::Reserved2(destination),
            _ => unreachable!("Invalid value for Flex message address."),
        }
    }
}

impl From<FlexAddress> for u8 {
    fn from(value: FlexAddress) -> Self {
        match value {
            FlexAddress::Channel(destination) => destination,
            FlexAddress::Group(destination) => 0x10 | destination,
            FlexAddress::Reserved1(reserved) => 0x20 | reserved,
            FlexAddress::Reserved2(reserved) => 0x30 | reserved,
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum FlexStatus {
    SetupAndPerformance(FlexSetupAndPerformance),
    MetadataText(FlexMetadataText),
    PerformanceTextEvent(FlexPerformanceTextEvent),
    Reserved(u16),
}

impl From<u16> for FlexStatus {
    fn from(value: u16) -> Self {
        let status = value.to_be_bytes();
        match status[0] {
            0x00 => FlexStatus::SetupAndPerformance(status[1].into()),
            0x01 => FlexStatus::MetadataText(status[1].into()),
            0x02 => FlexStatus::PerformanceTextEvent(status[1].into()),
            _ => FlexStatus::Reserved(value),
        }
    }
}

impl From<FlexStatus> for u16 {
    fn from(value: FlexStatus) -> Self {
        match value {
            FlexStatus::SetupAndPerformance(status) => u8::from(status) as u16,
            FlexStatus::MetadataText(status) => 0x01_00 | u8::from(status) as u16,
            FlexStatus::PerformanceTextEvent(status) => 0x02_00 | u8::from(status) as u16,
            FlexStatus::Reserved(bank_and_status) => bank_and_status,
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum FlexSetupAndPerformance {
    SetTempo,
    SetTimeSignature,
    SetMetronome,
    SetKeySignature,
    SetChordName,
    TextMessageCommonFormat(u8),
    Undefined(u8),
}

impl From<u8> for FlexSetupAndPerformance {
    fn from(value: u8) -> Self {
        debug_assert!(
            value < 16,
            "Wrong integer size: FlexSetupAndPerformance is u4"
        );
        match value {
            0x00 => Self::SetTempo,
            0x01 => Self::SetTimeSignature,
            0x02 => Self::SetMetronome,
            0x05 => Self::SetKeySignature,
            0x06 => Self::SetChordName,
            0x07..=0x0F => Self::TextMessageCommonFormat(value),
            0x03 | 0x04 => Self::Undefined(value),
            _ => unreachable!("Wrong integer size: FlexSetupAndPerformance is u4"),
        }
    }
}

impl From<FlexSetupAndPerformance> for u8 {
    fn from(value: FlexSetupAndPerformance) -> Self {
        match value {
            FlexSetupAndPerformance::SetTempo => 0x00,
            FlexSetupAndPerformance::SetTimeSignature => 0x01,
            FlexSetupAndPerformance::SetMetronome => 0x02,
            FlexSetupAndPerformance::SetKeySignature => 0x05,
            FlexSetupAndPerformance::SetChordName => 0x06,
            FlexSetupAndPerformance::TextMessageCommonFormat(val) => val,
            FlexSetupAndPerformance::Undefined(val) => val,
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
        debug_assert!(value < 16, "Wrong integer size: FlexMetadataText is u4");
        match value {
            0x0 => Self::Unknown,
            0x1 => Self::ProjectName,
            0x2 => Self::CompositionName,
            0x3 => Self::MidiClipName,
            0x4 => Self::CopyrightNotice,
            0x5 => Self::ComposerName,
            0x6 => Self::LyricistName,
            0x7 => Self::ArrangerName,
            0x8 => Self::PublisherName,
            0x9 => Self::PrimaryPerformerName,
            0xA => Self::AccompanyingPerformerName,
            0xB => Self::RecordingDate,
            0xC => Self::RecordingLocation,
            _ => unreachable!("Integer out of bounds: FlexMetadataText is u4 smaller than 0xD"),
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
            FlexMetadataText::AccompanyingPerformerName => 0xA,
            FlexMetadataText::RecordingDate => 0xB,
            FlexMetadataText::RecordingLocation => 0xC,
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum FlexPerformanceTextEvent {
    Unknown,
    /// Contains Lyrics as Unicode UTF-8 text.
    Lyrics,

    /// Contains a BCP 47 language identifier.
    LyricsLanguage,

    /// Contains Ruby lyrics as Unicode UTF-8 text, including [ruby characters](https://en.wikipedia.org/wiki/Ruby_character).
    RubyLyrics,

    /// Contains a BCP 47 language identifier.
    RubyLyricsLanguage,
}

impl From<u8> for FlexPerformanceTextEvent {
    fn from(value: u8) -> Self {
        debug_assert!(
            value < 16,
            "Wrong integer size: FlexPerformanceTextEvent is u4"
        );
        match value {
            0 => Self::Unknown,
            1 => Self::Lyrics,
            2 => Self::LyricsLanguage,
            3 => Self::RubyLyrics,
            4 => Self::RubyLyricsLanguage,
            _ => unreachable!(
                "Integer out of bounds: FlexPerformanceTextEvent is u4 smaller than 0x5"
            ),
        }
    }
}

impl From<FlexPerformanceTextEvent> for u8 {
    fn from(value: FlexPerformanceTextEvent) -> Self {
        match value {
            FlexPerformanceTextEvent::Unknown => 0,
            FlexPerformanceTextEvent::Lyrics => 1,
            FlexPerformanceTextEvent::LyricsLanguage => 2,
            FlexPerformanceTextEvent::RubyLyrics => 3,
            FlexPerformanceTextEvent::RubyLyricsLanguage => 4,
        }
    }
}

/// Declares and sets musical tempo in 10 nanosecond units per quarter note.
/// Address must be Group.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct FlexTempo(pub u32);

impl From<u32> for FlexTempo {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<FlexTempo> for u32 {
    fn from(value: FlexTempo) -> Self {
        value.0
    }
}

impl FlexTempo {
    pub fn bpm(&self) -> f32 {
        60.0 * 1_000_000.0 / self.0 as f32
    }

    pub fn from_bpm(bpm: f32) -> Self {
        Self((60.0 * 1_000_000.0 / bpm) as u32)
    }
}

/// Declares and sets a Time Signature for subsequent bars.
/// Address must be Group.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct FlexTimeSignature {
    /// A value from 1 to 256, supporting up to 256 beats in a bar.
    pub numerator: u8,

    /// A value in negative power of 2 (2 represents a quarter note, 3 represents an eighth note, etc.) If the value is set to zero, there is a non-standard denominator.
    pub denominator: u8,

    /// Expresses the number of 1/32 notes in 24 MIDI Clocks.
    pub number_of_32n: u8,
}

impl From<u32> for FlexTimeSignature {
    fn from(value: u32) -> Self {
        let bytes = value.to_ne_bytes();
        Self {
            numerator: bytes[0],
            denominator: bytes[1],
            number_of_32n: bytes[2],
        }
    }
}

impl From<FlexTimeSignature> for u32 {
    fn from(value: FlexTimeSignature) -> Self {
        u32::from_le_bytes([value.numerator, value.denominator, value.number_of_32n, 0])
    }
}

/// Sets metronome functions.
/// Address must be Group.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct FlexMetronome {
    /// Number of MIDI Clocks per Primary Click. 24 ticks per quarter note. (1 tick is 96n)
    /// Clicks would typically repeat at the value set by `SetTempo` message.
    pub clocks_per_primary_click: u8,

    /// Sets the number of Primary Clicks between each accent.
    /// for example, 5/4 divided into 3 + 2 will be represented as `[3, 2, 0]`
    pub bar_accents: [u8; 3],

    /// Declares the number of clicks that will sound within the period of a Primary Click.
    /// The two fields are independent, allowing for overlapping subdivision clicks.
    pub subdivision_clicks: [u8; 2],
}

impl From<[u32; 2]> for FlexMetronome {
    fn from(value: [u32; 2]) -> Self {
        let bytes = [value[0].to_ne_bytes(), value[1].to_ne_bytes()];

        FlexMetronome {
            clocks_per_primary_click: bytes[0][0],
            bar_accents: [bytes[0][1], bytes[0][2], bytes[0][3]],
            subdivision_clicks: [bytes[1][0], bytes[1][1]],
        }
    }
}

impl From<FlexMetronome> for [u32; 2] {
    fn from(value: FlexMetronome) -> Self {
        [
            u32::from_le_bytes([
                value.clocks_per_primary_click,
                value.bar_accents[0],
                value.bar_accents[1],
                value.bar_accents[2],
            ]),
            u32::from_le_bytes([
                value.subdivision_clicks[0],
                value.subdivision_clicks[1],
                0,
                0,
            ]),
        ]
    }
}

impl From<FlexMetronome> for [u32; 3] {
    fn from(value: FlexMetronome) -> Self {
        let smaller: [u32; 2] = value.into();
        [smaller[0], smaller[1], 0]
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct FlexKeySignature(pub(crate) u8);

impl FlexKeySignature {
    /// The number of sharps(+) or flats(-).
    pub fn sharps_or_flats(&self) -> i8 {
        let val = self.0 >> 4;
        val as i8 - if val < 8 { 0 } else { 16 }
    }
    /// The tonic note (A = 1, G = 7, others are non-standard)
    pub fn tonic_note(&self) -> u8 {
        self.0 & 0x0F
    }
}

impl From<u8> for FlexKeySignature {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<FlexKeySignature> for u8 {
    fn from(value: FlexKeySignature) -> Self {
        value.0
    }
}

impl FlexKeySignature {
    pub fn from_note(tonic_note: u8, sharps_or_flats: i8) -> Self {
        Self((sharps_or_flats as u8) << 4 | tonic_note)
    }
}

/// unimplemented
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct FlexChordName {
    pub tonic_note: NoteName,

    /// Positive values declare the number of sharps applied to the tonic note.
    /// Negative values declare the number of flats applied to the tonic note.
    pub tonic_alteration: i8,

    pub chord_type: ChordType,

    pub chord_alterations: [Alteration; 4],

    pub bass_note: NoteName,

    /// Positive values declare the number of sharps applied to the tonic note.
    /// Negative values declare the number of flats applied to the tonic note.
    /// if `-8`, Bass is the same as the chord tonic note.
    pub bass_alteration: i8,
    pub bass_chord_type: ChordType,
    pub bass_chord_alterations: [Alteration; 2],
}

impl From<[u32; 3]> for FlexChordName {
    fn from(value: [u32; 3]) -> Self {
        let bytes = [
            value[0].to_ne_bytes(),
            value[1].to_ne_bytes(),
            value[2].to_ne_bytes(),
        ];

        FlexChordName {
            tonic_alteration: i8_from_u4(bytes[0][0] >> 4),
            tonic_note: (bytes[0][0] & 0xF).into(),
            chord_type: (bytes[0][1]).into(),
            chord_alterations: [
                bytes[0][2].into(),
                bytes[0][3].into(),
                bytes[1][0].into(),
                bytes[1][1].into(),
            ],
            bass_alteration: i8_from_u4(bytes[2][0] >> 4),
            bass_note: (bytes[2][0] & 0xF).into(),
            bass_chord_type: bytes[2][1].into(),
            bass_chord_alterations: [bytes[2][2].into(), bytes[2][3].into()],
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum NoteName {
    Unknown,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    Reserved(u8),
}

impl From<u8> for NoteName {
    fn from(value: u8) -> Self {
        debug_assert!(value < 16, "Wrong integer size: NoteName is u4");
        match value {
            0x0 => Self::Unknown,
            0x1 => Self::A,
            0x2 => Self::B,
            0x3 => Self::C,
            0x4 => Self::D,
            0x5 => Self::E,
            0x6 => Self::F,
            0x7 => Self::G,
            _ => Self::Reserved(value),
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum ChordType {
    None,
    Major,
    Major6,
    Major7,
    Major9,
    Major11,
    Major13,
    Minor,
    Minor6,
    Minor7,
    Minor9,
    Minor11,
    Minor13,
    Dominant,
    Dominant9,
    Dominant11,
    Dominant13,
    Augmented,
    Augmented7,
    Diminished,
    Diminished7,
    HalfDiminished,
    MajorMinor,
    Pedal,
    Power,
    Suspended2,
    Suspended4,
    Suspended4_7,
    Reserved(u8),
}

impl From<u8> for ChordType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::None,
            0x01 => Self::Major,
            0x02 => Self::Major6,
            0x03 => Self::Major7,
            0x04 => Self::Major9,
            0x05 => Self::Major11,
            0x06 => Self::Major13,
            0x07 => Self::Minor,
            0x08 => Self::Minor6,
            0x09 => Self::Minor7,
            0x0A => Self::Minor9,
            0x0B => Self::Minor11,
            0x0C => Self::Minor13,
            0x0D => Self::Dominant,
            0x0E => Self::Dominant9,
            0x0F => Self::Dominant11,
            0x10 => Self::Dominant13,
            0x11 => Self::Augmented,
            0x12 => Self::Augmented7,
            0x13 => Self::Diminished,
            0x14 => Self::Diminished7,
            0x15 => Self::HalfDiminished,
            0x16 => Self::MajorMinor,
            0x17 => Self::Pedal,
            0x18 => Self::Power,
            0x19 => Self::Suspended2,
            0x1A => Self::Suspended4,
            0x1B => Self::Suspended4_7,
            _ => Self::Reserved(value),
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum Alteration {
    None,
    AddDegree(u8),
    SubtractDegree(u8),
    RaiseDegree(u8),
    LowerDegree(u8),
    Reserved { alteration_type: u8, degree: u8 },
}

impl From<u8> for Alteration {
    fn from(value: u8) -> Self {
        let alteration_type = value >> 4;
        let degree = value & 0xF;
        match alteration_type {
            0 => Self::None,
            1 => Self::AddDegree(degree),
            2 => Self::SubtractDegree(degree),
            3 => Self::RaiseDegree(degree),
            4 => Self::LowerDegree(degree),
            _ => Self::Reserved {
                alteration_type,
                degree,
            },
        }
    }
}

fn i8_from_u4(n: u8) -> i8 {
    if n < 8 {
        n as i8
    } else {
        n as i8 - 16
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    mod header {
        use super::*;

        #[test]
        fn address_from_u8() {
            assert_eq!(FlexAddress::Channel(0), 0x00_u8.into());
            assert_eq!(FlexAddress::Channel(0xF), 0x0F_u8.into());
            assert_eq!(FlexAddress::Group(0), 0x10_u8.into());
            assert_eq!(FlexAddress::Group(0xF), 0x1F_u8.into());
            assert_eq!(FlexAddress::Reserved1(0x0), 0x20_u8.into());
            assert_eq!(FlexAddress::Reserved1(0xF), 0x2F_u8.into());
            assert_eq!(FlexAddress::Reserved2(0xF), 0x3F_u8.into());
            assert_eq!(FlexAddress::Reserved2(0xF), 0x3F_u8.into());
        }

        #[test]
        fn address_from_u4_overflow() {
            let _a: FlexAddress = 0x40u8.into();
        }

        #[test]
        fn setup_and_performance_from_u8() {
            assert_eq!(FlexSetupAndPerformance::SetTempo, 0x00_u8.into());
            assert_eq!(FlexSetupAndPerformance::SetTimeSignature, 0x01_u8.into());
            assert_eq!(FlexSetupAndPerformance::SetMetronome, 0x02_u8.into());
            assert_eq!(FlexSetupAndPerformance::SetKeySignature, 0x05_u8.into());
            assert_eq!(FlexSetupAndPerformance::SetChordName, 0x06_u8.into());
            assert_eq!(
                FlexSetupAndPerformance::TextMessageCommonFormat(0x07),
                0x07_u8.into()
            );
            assert_eq!(
                FlexSetupAndPerformance::TextMessageCommonFormat(0x0F),
                0x0F_u8.into()
            );

            assert_eq!(FlexSetupAndPerformance::Undefined(0x03), 0x03_u8.into());
            assert_eq!(FlexSetupAndPerformance::Undefined(0x04), 0x04_u8.into());
        }

        #[test]
        #[should_panic]
        fn setup_and_performance_from_u8_overflow() {
            let _: FlexSetupAndPerformance = 0x10.into();
        }

        #[test]
        fn setup_and_performance_to_u8() {
            assert_eq!(0x00_u8, FlexSetupAndPerformance::SetTempo.into());
            assert_eq!(0x01_u8, FlexSetupAndPerformance::SetTimeSignature.into());
            assert_eq!(0x02_u8, FlexSetupAndPerformance::SetMetronome.into());
            assert_eq!(0x05_u8, FlexSetupAndPerformance::SetKeySignature.into());
            assert_eq!(0x06_u8, FlexSetupAndPerformance::SetChordName.into());
            assert_eq!(
                0x07_u8,
                FlexSetupAndPerformance::TextMessageCommonFormat(0x07).into()
            );
            assert_eq!(
                0x0F_u8,
                FlexSetupAndPerformance::TextMessageCommonFormat(0x0F).into()
            );

            assert_eq!(0x03_u8, FlexSetupAndPerformance::Undefined(0x03).into());
            assert_eq!(0x04_u8, FlexSetupAndPerformance::Undefined(0x04).into());
        }

        #[test]
        fn metadata_text_from_u8() {
            assert_eq!(FlexMetadataText::Unknown, 0x0u8.into());
            assert_eq!(FlexMetadataText::ProjectName, 0x1u8.into());
            assert_eq!(FlexMetadataText::CompositionName, 0x2u8.into());
            assert_eq!(FlexMetadataText::MidiClipName, 0x3u8.into());
            assert_eq!(FlexMetadataText::CopyrightNotice, 0x4u8.into());
            assert_eq!(FlexMetadataText::ComposerName, 0x5u8.into());
            assert_eq!(FlexMetadataText::LyricistName, 0x6u8.into());
            assert_eq!(FlexMetadataText::ArrangerName, 0x7u8.into());
            assert_eq!(FlexMetadataText::PublisherName, 0x8u8.into());
            assert_eq!(FlexMetadataText::PrimaryPerformerName, 0x9u8.into());
            assert_eq!(FlexMetadataText::AccompanyingPerformerName, 0xAu8.into());
            assert_eq!(FlexMetadataText::RecordingDate, 0xBu8.into());
            assert_eq!(FlexMetadataText::RecordingLocation, 0xCu8.into());
        }

        #[test]
        #[should_panic]
        fn metadata_text_from_overflow() {
            let _: FlexMetadataText = 0xDu8.into();
        }

        #[test]
        fn metadata_text_to_u8() {
            assert_eq!(0x0_u8, FlexMetadataText::Unknown.into());
            assert_eq!(0x1_u8, FlexMetadataText::ProjectName.into());
            assert_eq!(0x2_u8, FlexMetadataText::CompositionName.into());
            assert_eq!(0x3_u8, FlexMetadataText::MidiClipName.into());
            assert_eq!(0x4_u8, FlexMetadataText::CopyrightNotice.into());
            assert_eq!(0x5_u8, FlexMetadataText::ComposerName.into());
            assert_eq!(0x6_u8, FlexMetadataText::LyricistName.into());
            assert_eq!(0x7_u8, FlexMetadataText::ArrangerName.into());
            assert_eq!(0x8_u8, FlexMetadataText::PublisherName.into());
            assert_eq!(0x9_u8, FlexMetadataText::PrimaryPerformerName.into());
            assert_eq!(0xA_u8, FlexMetadataText::AccompanyingPerformerName.into());
            assert_eq!(0xB_u8, FlexMetadataText::RecordingDate.into());
            assert_eq!(0xC_u8, FlexMetadataText::RecordingLocation.into());
        }

        #[test]
        fn performance_text_from_u8() {
            assert_eq!(FlexPerformanceTextEvent::Unknown, 0u8.into());
            assert_eq!(FlexPerformanceTextEvent::Lyrics, 1u8.into());
            assert_eq!(FlexPerformanceTextEvent::LyricsLanguage, 2u8.into());
            assert_eq!(FlexPerformanceTextEvent::RubyLyrics, 3u8.into());
            assert_eq!(FlexPerformanceTextEvent::RubyLyricsLanguage, 4u8.into());
        }

        #[test]
        #[should_panic]
        fn performance_text_from_overflow() {
            let _: FlexPerformanceTextEvent = 5.into();
        }

        #[test]
        fn performance_text_into_u8() {
            assert_eq!(0_u8, FlexPerformanceTextEvent::Unknown.into());
            assert_eq!(1_u8, FlexPerformanceTextEvent::Lyrics.into());
            assert_eq!(2_u8, FlexPerformanceTextEvent::LyricsLanguage.into());
            assert_eq!(3_u8, FlexPerformanceTextEvent::RubyLyrics.into());
            assert_eq!(4_u8, FlexPerformanceTextEvent::RubyLyricsLanguage.into());
        }

        #[test]
        fn status_from_u16() {
            assert_eq!(
                FlexStatus::SetupAndPerformance(FlexSetupAndPerformance::SetTempo),
                u16::from_le(0x0000_u16).into()
            );
            assert_eq!(
                FlexStatus::MetadataText(0.into()),
                u16::from_le(0x0100_u16).into()
            );
            assert_eq!(
                FlexStatus::PerformanceTextEvent(0.into()),
                u16::from_le(0x0200_u16).into()
            );
            assert_eq!(
                FlexStatus::Reserved(u16::from_le(0x03_00)),
                u16::from_le(0x0300_u16).into()
            );
        }

        #[cfg(target_endian = "little")]
        #[test]
        fn status_from_u4_status_bank_overflow() {
            let _a = FlexStatus::from(0x0400_u16);
        }

        #[test]
        fn status_into_u16() {
            assert_eq!(
                0x0000_u16,
                FlexStatus::SetupAndPerformance(FlexSetupAndPerformance::SetTempo).into()
            );
            assert_eq!(0x0100_u16, FlexStatus::MetadataText(0.into()).into());
            assert_eq!(
                0x0200_u16,
                FlexStatus::PerformanceTextEvent(0.into()).into()
            );
            assert_eq!(0x0300_u16, FlexStatus::Reserved(0x0300).into());
        }
    }

    #[test]
    fn tempo_from_u32() {
        assert_eq!(FlexTempo(0), 0u32.into());
    }

    #[test]
    fn tempo_into_u32() {
        assert_eq!(0u32, FlexTempo(0).into());
    }

    #[test]
    fn tempo_bpm() {
        assert_eq!(120.0, FlexTempo(500_000).bpm());
        assert_eq!(FlexTempo(500_000), FlexTempo::from_bpm(120.0));
    }

    #[cfg(target_endian = "little")]
    #[test]
    fn time_signature_from_u32() {
        assert_eq!(
            FlexTimeSignature {
                numerator: 0,
                denominator: 0,
                number_of_32n: 0,
            },
            0x0000_0000u32.into()
        );
        assert_eq!(
            FlexTimeSignature {
                numerator: 0xA0,
                denominator: 0xB1,
                number_of_32n: 0xC2,
            },
            u32::from_le(0x00C2_B1A0_u32).into()
        );
    }

    #[test]
    fn time_signature_into_u32() {
        assert_eq!(
            0x0000_0000u32,
            FlexTimeSignature {
                numerator: 0,
                denominator: 0,
                number_of_32n: 0,
            }
            .into()
        );
        assert_eq!(
            u32::from_le(0x00C2_B1A0),
            FlexTimeSignature {
                numerator: 0xA0,
                denominator: 0xB1,
                number_of_32n: 0xC2,
            }
            .into()
        );
    }

    #[test]
    fn metronome_from_u32_2() {
        assert_eq!(
            FlexMetronome {
                clocks_per_primary_click: 0xA1,
                bar_accents: [0xB2, 0xC3, 0xD4],
                subdivision_clicks: [0xE5, 0xF6],
            },
            [u32::from_le(0xD4C3_B2A1), u32::from_le(0x0000_F6E5)].into()
        );
    }

    #[test]
    fn metronome_into_u32_array() {
        let a: [u32; 2] = FlexMetronome {
            clocks_per_primary_click: 0xA1,
            bar_accents: [0xB2, 0xC3, 0xD4],
            subdivision_clicks: [0xE5, 0xF6],
        }
        .into();
        assert_eq!(
            [u32::from_le(0xD4C3_B2A1_u32), u32::from_le(0x0000_F6E5)],
            a
        );

        let b: [u32; 3] = FlexMetronome {
            clocks_per_primary_click: 0xA1,
            bar_accents: [0xB2, 0xC3, 0xD4],
            subdivision_clicks: [0xE5, 0xF6],
        }
        .into();

        assert_eq!([u32::from_le(0xD4C3_B2A1), u32::from_le(0x0000_F6E5), 0], b);
    }

    #[test]
    fn key_signature_from_u8() {
        let key_sig: FlexKeySignature = 0x00.into();
        assert_eq!(0, key_sig.sharps_or_flats());
        assert_eq!(0, key_sig.tonic_note());

        let key_sig: FlexKeySignature = 0x21.into();
        assert_eq!(2, key_sig.sharps_or_flats());
        assert_eq!(1, key_sig.tonic_note());

        let key_sig: FlexKeySignature = 0xE7.into();
        assert_eq!(-2, key_sig.sharps_or_flats());
        assert_eq!(7, key_sig.tonic_note());
    }

    #[test]
    fn key_signature_from_note() {
        let key_sig = FlexKeySignature::from_note(3, 2);
        assert_eq!(0x23, key_sig.0);

        let key_sig = FlexKeySignature::from_note(3, -2);
        assert_eq!(0xE3, key_sig.0);
    }

    #[test]
    fn key_signature_into_u8() {
        let key_sig: FlexKeySignature = 0x00.into();
        let num: u8 = key_sig.into();
        assert_eq!(0, num);

        let key_sig: FlexKeySignature = 0x21.into();
        let num: u8 = key_sig.into();
        assert_eq!(0x21, num);

        let key_sig: FlexKeySignature = 0xE7.into();
        let num: u8 = key_sig.into();
        assert_eq!(0xE7, num);
    }

    #[test]
    fn chord_name_from_u32_3() {
        let chord = FlexChordName {
            tonic_note: 0x1.into(),
            tonic_alteration: i8_from_u4(0xA),
            chord_type: 0xB2.into(),
            chord_alterations: [0xC3.into(), 0xD4.into(), 0xE5.into(), 0xF6.into()],
            bass_note: 0.into(),
            bass_alteration: i8_from_u4(0xF),
            bass_chord_type: 0x1A.into(),
            bass_chord_alterations: [0x2B.into(), 0x3C.into()],
        };
        let source = [
            u32::from_le(0xD4C3_B2A1_u32),
            u32::from_le(0x_0000_F6E5),
            u32::from_le(0x3C2B_1AF0),
        ];
        assert_eq!(chord, source.into());
    }
}
