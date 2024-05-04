//! Helpers for encoding and decoding UMPs.
use core::convert::TryInto;

use crate::{message::*, packet::*};

/// Write a message to an output buffer.
pub fn encode(message: impl Message, buf: &mut [u8]) -> usize {
    4 * message
        .iter()
        .zip(buf.chunks_exact_mut(core::mem::size_of::<u32>()))
        .map(|(word, chunk)| chunk.copy_from_slice(&word.to_ne_bytes()))
        .count()
}

/// Read a message from an output buffer.
pub fn decode(buf: &[u8]) -> Option<(usize, Data)> {
    let mut chunks = buf.chunks_exact(core::mem::size_of::<u32>());
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
            Some((4, data))
        }
        0x3 | 0x4 | 0x8 | 0x9 | 0xa => {
            let word1 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
            let packet = Packet::<2>([word0, word1]);
            let data = match message_type {
                0x3 => Data::Data64(data::Data64::from_packet_unchecked(packet)),
                0x4 => Data::ChannelVoice(channel2::ChannelVoice::from_packet_unchecked(packet)),
                _ => Data::Reserved64(packet),
            };
            Some((8, data))
        }
        0xb | 0xc => {
            let word1 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
            let word2 = u32::from_ne_bytes(chunks.next()?.try_into().ok()?);
            let packet = Packet::<3>([word0, word1, word2]);
            Some((12, Data::Reserved96(packet)))
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
            Some((16, data))
        }
        _ => unreachable!("Invalid message type."),
    }
}
