//! Implements serializing and deserializing MIDI messages as universal midi packets (UMP)
use core::ops::Deref;

/// A universal midi packet (UMP) is a 32, 64, 96, or 128 bit slice of serialized
/// MIDI data that is parsed into midi messages, or serialized from them.
///
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Packet<const N: usize>(pub(crate) [u32; N]);

impl<const N: usize> Deref for Packet<N> {
    type Target = [u32];
    fn deref(&self) -> &'_ Self::Target {
        &self.0
    }
}

impl<const N: usize> Packet<N> {
    pub fn message_type(&self) -> u8 {
        self.0[0].to_ne_bytes()[0] >> 4
    }

    pub fn group(&self) -> u8 {
        self.0[0].to_ne_bytes()[0] & 0x0f
    }

    pub fn status(&self) -> u8 {
        // Lead
        self.0[0].to_ne_bytes()[1]
    }
}

pub type Packet32 = Packet<1>;
pub type Packet64 = Packet<2>;
pub type Packet96 = Packet<3>;
pub type Packet128 = Packet<4>;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum MessageType {
    Utility = 0,
    System = 1,
    LegacyChannelVoice = 2,
    Data64 = 3,
    ChannelVoice = 4,
    Data128 = 5,
    Reserved6 = 6,
    Reserved7 = 7,
    Reserved8 = 8,
    Reserved9 = 9,
    Reserved10 = 0xA,
    Reserved11 = 0xB,
    Reserved12 = 0xC,
    Flex = 0xD,
    Reserved14 = 0xE,
    UmpStream = 0xF,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageType::Utility,
            1 => MessageType::System,
            2 => MessageType::LegacyChannelVoice,
            3 => MessageType::Data64,
            4 => MessageType::ChannelVoice,
            5 => MessageType::Data128,
            6 => MessageType::Reserved6,
            7 => MessageType::Reserved7,
            8 => MessageType::Reserved8,
            9 => MessageType::Reserved9,
            10 => MessageType::Reserved10,
            11 => MessageType::Reserved11,
            12 => MessageType::Reserved12,
            13 => MessageType::Flex,
            14 => MessageType::Reserved14,
            15 => MessageType::UmpStream,
            _ => unreachable!("Invalid value for message type."),
        }
    }
}
