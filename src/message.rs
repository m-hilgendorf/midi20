//! MIDI 2.0 Message Types
//!
//! MIDI 2.0 Messages form a mostly flat abstract syntax tree. MIDI 1.0 types are represented by
//! the [LegacyChannelVoice] enum.

use core::ops::Deref;

pub use data::{Data128, Data64, DataStatus};
pub use flex::{
    Flex, FlexAddress, FlexChordName, FlexKeySignature, FlexMetronome, FlexStatus,
    FlexTimeSignature,
};
pub use legacy_voice::{LegacyChannelVoice, LegacyChannelVoiceStatus};
pub use system::{System, SystemStatus};
pub use utility::{Utility, UtilityStatus};
pub use voice::{ChannelVoice, ChannelVoiceStatus};

use crate::message::ump_stream::UmpStream;
use crate::packet::*;

pub mod data;
pub mod flex;
pub mod legacy_voice;
pub mod system;
pub mod ump_stream;
pub mod utility;
pub mod voice;

pub trait Message
    where
        Self: Deref<Target=[u32]>,
{
    type Status;
    type Data;

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
    /// System real time types (start, stop, continue, timing clock, active sensing, reset)
    System(System),

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

    /// Ump Stream messages
    UmpStream(UmpStream),

    Reserved32(Packet32),
    Reserved64(Packet64),
    Reserved96(Packet96),
    Reserved128(Packet128),
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum DataFormat {
    SinglePacket,
    Start,
    Continue,
    End,
    Reserved,
}

impl From<u8> for DataFormat {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::SinglePacket,
            1 => Self::Start,
            2 => Self::Continue,
            3 => Self::End,
            _ => Self::Reserved,
        }
    }
}

impl From<DataFormat> for u8 {
    fn from(value: DataFormat) -> Self {
        match value {
            DataFormat::SinglePacket => 0,
            DataFormat::Start => 1,
            DataFormat::Continue => 2,
            DataFormat::End => 3,
            DataFormat::Reserved => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum ExtendedDataFormat {
    SinglePacket,
    Start,
    Continue,
    End,
    MixedDataSetHeader,
    MixedDataSetPayload,
    Reserved,
}

impl From<u8> for ExtendedDataFormat {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::SinglePacket,
            1 => Self::Start,
            2 => Self::Continue,
            3 => Self::End,
            8 => Self::MixedDataSetHeader,
            9 => Self::MixedDataSetPayload,
            _ => Self::Reserved,
        }
    }
}

impl From<ExtendedDataFormat> for u8 {
    fn from(value: ExtendedDataFormat) -> Self {
        match value {
            ExtendedDataFormat::SinglePacket => 0,
            ExtendedDataFormat::Start => 1,
            ExtendedDataFormat::Continue => 2,
            ExtendedDataFormat::End => 3,
            ExtendedDataFormat::MixedDataSetHeader => 8,
            ExtendedDataFormat::MixedDataSetPayload => 9,
            ExtendedDataFormat::Reserved => unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::message::{DataFormat, ExtendedDataFormat};

    #[test]
    fn format_from_u8() {
        assert_eq!(DataFormat::SinglePacket, 0u8.into());
        assert_eq!(DataFormat::Start, 1u8.into());
        assert_eq!(DataFormat::Continue, 2u8.into());
        assert_eq!(DataFormat::End, 3u8.into());
        assert_eq!(DataFormat::Reserved, 4u8.into());
    }

    #[test]
    fn extended_format_from_u8() {
        assert_eq!(ExtendedDataFormat::SinglePacket, 0u8.into());
        assert_eq!(ExtendedDataFormat::Start, 1u8.into());
        assert_eq!(ExtendedDataFormat::Continue, 2u8.into());
        assert_eq!(ExtendedDataFormat::End, 3u8.into());
        assert_eq!(ExtendedDataFormat::MixedDataSetHeader, 8u8.into());
        assert_eq!(ExtendedDataFormat::MixedDataSetPayload, 9u8.into());
    }
}