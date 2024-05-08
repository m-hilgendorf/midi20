//! MIDI 2.0 Message Types
//!
//! MIDI 2.0 Messages form a mostly flat abstract syntax tree. MIDI 1.0 types are represented by
//! the [LegacyChannelVoice] enum.
use crate::packet::*;
use std::{convert::TryInto, mem, ops::Deref, slice};

pub mod channel1;
pub mod channel2;
pub mod data;
pub mod flex;
pub mod system;
pub mod ump_stream;
pub mod utility;

/// A shared trait by all MIDI messages.
pub trait Message
where
    Self: Deref<Target = [u32]>,
{
    /// A rich type representation of this message's data.
    type Data;

    /// A rich type representation of this message's status.
    type Status;

    /// The `mt` nibble of the message (most significant four bits).
    fn message_type(&self) -> MessageType;

    /// The group that this message is destined for.
    fn group(&self) -> u8;

    /// The status information of this message, eg NoteOn.
    fn status(&self) -> Self::Status;

    /// The data contents of this message.
    fn data(&self) -> Self::Data;
}

/// Parent type of each MIDI Message.
#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub enum Data {
    /// Utility types (Jitter reduction, Clock, No-op).
    Utility(utility::Utility),

    /// System common types (midi time code, song position, tune request, song number).
    System(system::System),

    /// MIDI 1.0 channel voice messages (note on/off, key pressure, program change, control change).
    LegacyChannelVoice(channel1::LegacyChannelVoice),

    /// MIDI 2.0 channel voice messages.
    ChannelVoice(channel2::ChannelVoice),

    /// Flex data messages: real time messages with limited variability of size.
    Flex(flex::Flex),

    /// UMP Stream.
    UmpStream(ump_stream::UmpStream),

    /// 64 bit data messages.
    Data64(data::Data64),

    /// 128 bit data messages.
    Data128(data::Data128),

    /// Reserved.
    Reserved32(Packet32),

    /// Reserved.
    Reserved64(Packet64),

    /// Reserved.
    Reserved96(Packet96),

    /// Reserved.
    Reserved128(Packet128),
}

impl Data {
    /// Parse a chunk of bytes into a message.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut chunks = bytes.chunks_exact(mem::size_of::<u32>());
        let word0 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
        let message_type = word0 >> 28;
        match message_type {
            0x0 | 0x1 | 0x2 | 0x6 | 0x7 => {
                let packet = Packet::<1>([word0]);
                let data = match message_type {
                    0x0 => Data::Utility(utility::Utility::from_packet_unchecked(packet)),
                    0x1 => Data::System(system::System::from_packet_unchecked(packet)),
                    0x2 => Data::LegacyChannelVoice(
                        channel1::LegacyChannelVoice::from_packet_unchecked(packet),
                    ),
                    _ => Data::Reserved32(packet),
                };
                Some(data)
            }
            0x3 | 0x4 | 0x8 | 0x9 | 0xa => {
                let word1 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
                let packet = Packet::<2>([word0, word1]);
                let data = match message_type {
                    0x3 => Data::Data64(data::Data64::from_packet_unchecked(packet)),
                    0x4 => {
                        Data::ChannelVoice(channel2::ChannelVoice::from_packet_unchecked(packet))
                    }
                    _ => Data::Reserved64(packet),
                };
                Some(data)
            }
            0xb | 0xc => {
                let word1 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
                let word2 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
                let packet = Packet::<3>([word0, word1, word2]);
                Some(Data::Reserved96(packet))
            }
            0x5 | 0xd | 0xe | 0xf => {
                let word1 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
                let word2 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
                let word3 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
                let packet = Packet::<4>([word0, word1, word2, word3]);
                let data = match message_type {
                    0x5 => Data::Data128(data::Data128::from_packet_unchecked(packet)),
                    0xd => Data::Flex(flex::Flex::from_packet_unchecked(packet)),
                    0xf => Data::UmpStream(ump_stream::UmpStream::from_packet_unchecked(packet)),
                    _ => Data::Reserved128(packet),
                };
                Some(data)
            }
            _ => unreachable!("Invalid message type."),
        }
    }

    /// Get the raw bytes of the message.
    pub fn as_bytes(&self) -> &[u8] {
        let words: &[u32] = match self {
            Self::Utility(msg) => &msg.0 .0,
            Self::System(msg) => &msg.0 .0,
            Self::LegacyChannelVoice(msg) => &msg.0 .0,
            Self::ChannelVoice(msg) => &msg.0 .0,
            Self::Flex(msg) => &msg.0 .0,
            Self::UmpStream(msg) => &msg.0 .0,
            Self::Data64(msg) => &msg.0 .0,
            Self::Data128(msg) => &msg.0 .0,
            Self::Reserved32(msg) => &msg.0,
            Self::Reserved64(msg) => &msg.0,
            Self::Reserved96(msg) => &msg.0,
            Self::Reserved128(msg) => &msg.0,
        };
        let data = words.as_ptr().cast();
        let len = words.len() * mem::size_of::<u32>();
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// The size of this message's packet in bytes.
    pub fn packet_size(&self) -> usize {
        self.as_bytes().len()
    }

    /// Get the group of the message.
    pub fn group(&self) -> u8 {
        match self {
            Self::Utility(msg) => msg.group(),
            Self::System(msg) => msg.group(),
            Self::LegacyChannelVoice(msg) => msg.group(),
            Self::ChannelVoice(msg) => msg.group(),
            Self::Flex(msg) => msg.group(),
            Self::UmpStream(msg) => msg.group(),
            Self::Data64(msg) => msg.group(),
            Self::Data128(msg) => msg.group(),
            Self::Reserved32(msg) => msg.group(),
            Self::Reserved64(msg) => msg.group(),
            Self::Reserved96(msg) => msg.group(),
            Self::Reserved128(msg) => msg.group(),
        }
    }

    /// Get the channel of the message, if it exists.
    pub fn channel(&self) -> Option<u8> {
        match self {
            Self::LegacyChannelVoice(msg) => Some(msg.channel()),
            Self::ChannelVoice(msg) => Some(msg.channel()),
            _ => None,
        }
    }
}
