//! MIDI 2.0 Message Types
//!
//! MIDI 2.0 Messages form a mostly flat abstract syntax tree. MIDI 1.0 types are represented by
//! the [LegacyChannelVoice] enum.
use crate::packet::*;
use core::ops::Deref;

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
pub enum MidiMessageData {
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
