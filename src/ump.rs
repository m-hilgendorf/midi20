//! Implements serializing and deserializing MIDI messages as universal midi packets (UMP)
use crate::message::{
    ChannelVoice, Data128, Data64, LegacyChannelVoice, MidiMessage, MidiMessageData, SystemCommon,
    SystemRealtime, Utility,
};
use core::borrow::Borrow;
use core::ops::Deref;

/// A universal midi packet (UMP) is a 32, 64, 96, or 128 bit slice of serialized
/// MIDI data that is parsed into midi messages, or serialized from them.
///
/// Usage: 
/// ```rust
/// use midi::{ump::UMP, message::*};
/// let cc: MidiMessage = ChannelVoice::ControlChange { channel: 0, control: 7, value: 64 }.into_message();
/// let ump: UMP = cc.into();
/// println!("{:?}", UMP); 
/// ```
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum UMP {
    U32([u8; 4]),
    U64([u8; 8]),
    U96([u8; 12]),
    U128([u8; 16]),
}

impl Deref for UMP {
    type Target = [u8];
    fn deref(&self) -> &'_ Self::Target {
        match self {
            UMP::U32(b) => b,
            UMP::U64(b) => b,
            UMP::U96(b) => b,
            UMP::U128(b) => b,
        }
    }
}

impl UMP {
    /// Creates an iterator of UMPs from an iterator of bytes.
    pub fn stream_from_bytes<B>(mut bytes: impl Iterator<Item = B>) -> impl Iterator<Item = UMP>
    where
        B: Borrow<u8>,
    {
        let mut next_byte = move || -> Option<u8> { Some(*bytes.next()?.borrow()) };
        core::iter::from_fn(move || {
            let msb = next_byte()?;
            let packet = match msb >> 4 {
                0x0 | 0x1 | 0x2 | 0x6 | 0x7 => {
                    let mut buf = [0; 4];
                    buf[0] = msb;
                    for i in 0..3 {
                        buf[i + 1] = next_byte()?;
                    }
                    UMP::U32(buf)
                }
                0x3 | 0x4 | 0x8 | 0x9 | 0xA => {
                    let mut buf = [0; 8];
                    buf[0] = msb;
                    for i in 0..7 {
                        buf[i + 1] = next_byte()?;
                    }
                    UMP::U64(buf)
                }
                0xB | 0xC => {
                    let mut buf = [0; 12];
                    buf[0] = msb;
                    for i in 0..11 {
                        buf[i + 1] = next_byte()?;
                    }
                    UMP::U96(buf)
                }
                0x5 | 0xD | 0xE | 0xF => {
                    let mut buf = [0; 16];
                    buf[0] = msb;
                    for i in 0..15 {
                        buf[i + 1] = next_byte()?;
                    }
                    UMP::U128(buf)
                }
                _ => return None,
            };
            Some(packet)
        })
    }
}

fn undefined(group: u8, ump: UMP) -> MidiMessage {
    MidiMessage {
        group,
        data: MidiMessageData::Undefined(ump),
    }
}

impl From<UMP> for MidiMessage {
    fn from(ump: UMP) -> MidiMessage {
        match ump {
            UMP::U32(bytes) => match bytes[0] >> 4 {
                0 => utility(bytes),
                1 => system(bytes),
                2 => channel_voice1(bytes),
                _ => undefined(bytes[0] & 0x0f, UMP::U32(bytes)),
            },
            UMP::U64(bytes) => match bytes[0] >> 4 {
                3 => data64(bytes),
                4 => channel_voice2(bytes),
                _ => undefined(bytes[0] & 0x0f, UMP::U64(bytes)),
            },
            UMP::U96(bytes) => undefined(bytes[0] & 0x0f, UMP::U96(bytes)),
            UMP::U128(bytes) => match bytes[0] >> 4 {
                5 => data128(bytes),
                _ => undefined(bytes[0] & 0x0f, UMP::U128(bytes)),
            },
        }
    }
}

impl From<MidiMessage> for UMP {
    fn from(msg: MidiMessage) -> UMP {
        let group = msg.group();
        let data = msg.data;

        match data {
            MidiMessageData::Undefined(ump) => ump,
            MidiMessageData::Data64(data) => UMP::U64(data.bytes()),
            MidiMessageData::Data128(data) => UMP::U128(data.bytes()),
            MidiMessageData::SystemCommon(sys) => match sys {
                SystemCommon::TimeCode(tc) => UMP::U32([0x10 | group, 0xF1, tc, 0]),
                SystemCommon::SongPositionPointer(spp) => {
                    let bytes = spp.to_be_bytes();
                    UMP::U32([0x10 | group, 0xF2, bytes[0], bytes[1]])
                }
                SystemCommon::SongSelect(ss) => UMP::U32([0x10 | group, 0xF3, ss, 0]),
                SystemCommon::TuneRequest => UMP::U32([0x10 | group, 0xF6, 0, 0]),
            },
            MidiMessageData::SystemRealtime(sys) => {
                let status = match sys {
                    SystemRealtime::Start => 0xFA,
                    SystemRealtime::Continue => 0xFC,
                    SystemRealtime::Stop => 0xFC,
                    SystemRealtime::TimingClock => 0xF8,
                    SystemRealtime::ActiveSensing => 0xFE,
                    SystemRealtime::Reset => 0xFF,
                };
                UMP::U32([0x10 | group, status, 0, 0])
            }
            MidiMessageData::LegacyChannelVoice(cv) => match cv {
                LegacyChannelVoice::NoteOff { channel, note, vel } => {
                    UMP::U32([0x20 | group, 0x80 | channel, note, vel])
                }
                LegacyChannelVoice::NoteOn { channel, note, vel } => {
                    UMP::U32([0x20 | group, 0x90 | channel, note, vel])
                }
                LegacyChannelVoice::PolyPressure {
                    channel,
                    note,
                    pressure,
                } => UMP::U32([0x20 | group, 0xA0 | channel, note, pressure]),
                LegacyChannelVoice::ControlChange {
                    channel,
                    control,
                    value,
                } => UMP::U32([0x20 | group, 0xB0 | channel, control, value]),
                LegacyChannelVoice::ProgramChange {
                    channel,
                    program,
                    _reserved,
                } => UMP::U32([0x20 | group, 0xC0 | channel, program, _reserved]),
                LegacyChannelVoice::ChannelPressure {
                    channel,
                    pressure,
                    _reserved,
                } => UMP::U32([0x20 | group, 0xD0 | channel, pressure, _reserved]),
                LegacyChannelVoice::PitchBend { channel, data } => {
                    let bytes = data.to_le_bytes();
                    UMP::U32([0x20 | group, 0xE0 | channel, bytes[0], bytes[1]])
                }
            },
            MidiMessageData::Utility(util) => match util {
                Utility::NoOp => UMP::U32([group, 0, 0, 0]),
                Utility::JrClock(clock) => {
                    let bytes = clock.to_le_bytes();
                    UMP::U32([group, 0x10, bytes[0], bytes[1]])
                }
                Utility::JrTimestamp(stamp) => {
                    let bytes = stamp.to_le_bytes();
                    UMP::U32([group, 0x20, bytes[0], bytes[1]])
                }
            },
            MidiMessageData::ChannelVoice(cv) => match cv {
                ChannelVoice::NoteOff {
                    channel,
                    note,
                    vel,
                    attr,
                    attr_val,
                } => {
                    let vel_bytes = vel.to_le_bytes();
                    let attr_val_bytes = attr_val.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x80 | channel,
                        note,
                        vel_bytes[0],
                        vel_bytes[1],
                        attr,
                        attr_val_bytes[0],
                        attr_val_bytes[1],
                    ])
                }
                ChannelVoice::NoteOn {
                    channel,
                    note,
                    vel,
                    attr,
                    attr_val,
                } => {
                    let vel_bytes = vel.to_le_bytes();
                    let attr_val_bytes = attr_val.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x90 | channel,
                        note,
                        vel_bytes[0],
                        vel_bytes[1],
                        attr,
                        attr_val_bytes[0],
                        attr_val_bytes[1],
                    ])
                }
                ChannelVoice::PolyPressure {
                    channel,
                    note,
                    data,
                } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0xA0 | channel,
                        note,
                        0,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::RegPerNoteCtrl {
                    channel,
                    note,
                    control,
                    data,
                } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x00 | channel,
                        note,
                        control,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::AsgnPerNoteCtrl {
                    channel,
                    note,
                    control,
                    data,
                } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x10 | channel,
                        note,
                        control,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::PerNoteMngmt {
                    channel,
                    note,
                    flags,
                } => UMP::U64([0x40 | group, 0xF0 | channel, note, flags, 0, 0, 0, 0]),
                ChannelVoice::ControlChange {
                    channel,
                    control,
                    data,
                } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0xC0 | channel,
                        control,
                        0,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::RegsteredCtrl {
                    channel,
                    bank,
                    index,
                    data,
                } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x20 | channel,
                        bank,
                        index,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::AssignableCtrl {
                    channel,
                    bank,
                    index,
                    data,
                } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x30 | channel,
                        bank,
                        index,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::RelRegCtrl {
                    channel,
                    bank,
                    index,
                    data,
                } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x40 | channel,
                        bank,
                        index,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::RelAssnCtrl {
                    channel,
                    bank,
                    index,
                    data,
                } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x50 | channel,
                        bank,
                        index,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::ProgramChange {
                    channel,
                    options,
                    program,
                    bank,
                } => {
                    let bytes = bank.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x90 | channel,
                        0,
                        options,
                        program,
                        0,
                        bytes[0],
                        bytes[1],
                    ])
                }
                ChannelVoice::ChannelPressure { channel, data } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0xC0 | channel,
                        0,
                        0,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::PitchBend { channel, data } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0xD0 | channel,
                        0,
                        0,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
                ChannelVoice::PerNotePitchBnd {
                    channel,
                    note,
                    data,
                } => {
                    let bytes = data.to_le_bytes();
                    UMP::U64([
                        0x40 | group,
                        0x60 | channel,
                        note,
                        0,
                        bytes[0],
                        bytes[1],
                        bytes[2],
                        bytes[3],
                    ])
                }
            },
        }
    }
}

fn channel_voice1(bytes: [u8; 4]) -> MidiMessage {
    let group = bytes[0] & 0x0f;
    let channel = bytes[1] & 0x0f;
    match bytes[1] >> 4 {
        0b1000 => {
            // Note OFF
            let note = bytes[2];
            let vel = bytes[3];
            MidiMessage {
                group,
                data: MidiMessageData::LegacyChannelVoice(LegacyChannelVoice::NoteOff {
                    channel,
                    note,
                    vel,
                }),
            }
        }
        0b1001 => {
            // Note ON
            let note = bytes[2];
            let vel = bytes[3];
            MidiMessage {
                group,
                data: MidiMessageData::LegacyChannelVoice(LegacyChannelVoice::NoteOn {
                    channel,
                    note,
                    vel,
                }),
            }
        }
        0b1010 => {
            // Poly Pressure
            let note = bytes[2];
            let pressure = bytes[3];
            MidiMessage {
                group,
                data: MidiMessageData::LegacyChannelVoice(LegacyChannelVoice::PolyPressure {
                    channel,
                    note,
                    pressure,
                }),
            }
        }
        0b1011 => {
            // CC
            let control = bytes[2];
            let value = bytes[3];
            MidiMessage {
                group,
                data: MidiMessageData::LegacyChannelVoice(LegacyChannelVoice::ControlChange {
                    channel,
                    control,
                    value,
                }),
            }
        }
        0b1100 => {
            // Program change
            let program = bytes[2];
            let _reserved = bytes[3];
            MidiMessage {
                group,
                data: MidiMessageData::LegacyChannelVoice(LegacyChannelVoice::ProgramChange {
                    channel,
                    program,
                    _reserved,
                }),
            }
        }
        0b1101 => {
            // Channel pressure
            let pressure = bytes[2];
            let _reserved = bytes[3];
            MidiMessage {
                group,
                data: MidiMessageData::LegacyChannelVoice(LegacyChannelVoice::ChannelPressure {
                    channel,
                    pressure,
                    _reserved,
                }),
            }
        }
        0b1110 => {
            let data = u16::from_le_bytes([bytes[2], bytes[3]]);
            MidiMessage {
                group,
                data: MidiMessageData::LegacyChannelVoice(LegacyChannelVoice::PitchBend {
                    channel,
                    data,
                }),
            }
        }
        _ => unreachable!(),
    }
}

fn channel_voice2(bytes: [u8; 8]) -> MidiMessage {
    let group = bytes[0] & 0x0f;
    let channel = bytes[1] & 0x0f;
    let status = bytes[1] >> 4;
    match status {
        0b1000 => {
            // Note OFF
            let note = bytes[2];
            let attr = bytes[3];
            let vel = u16::from_le_bytes([bytes[4], bytes[5]]);
            let attr_val = u16::from_le_bytes([bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::NoteOff {
                    channel,
                    note,
                    attr,
                    attr_val,
                    vel,
                }),
            }
        }
        0b1001 => {
            // Note ON
            let note = bytes[2];
            let attr = bytes[3];
            let vel = u16::from_le_bytes([bytes[4], bytes[5]]);
            let attr_val = u16::from_le_bytes([bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::NoteOn {
                    channel,
                    note,
                    attr,
                    attr_val,
                    vel,
                }),
            }
        }
        0b1010 => {
            // Poly Pressure
            let note = bytes[2];
            let _reserved = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::PolyPressure {
                    channel,
                    note,
                    data,
                }),
            }
        }
        0b0000 => {
            // Registered Per-Note controller
            let note = bytes[2];
            let control = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::RegPerNoteCtrl {
                    channel,
                    note,
                    control,
                    data,
                }),
            }
        }
        0b0001 => {
            // Assignable Per-Note controller
            let note = bytes[2];
            let control = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::AsgnPerNoteCtrl {
                    channel,
                    note,
                    control,
                    data,
                }),
            }
        }
        0b1111 => {
            // Per-note management message
            let note = bytes[2];
            let flags = bytes[3];
            let _reserved = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::PerNoteMngmt {
                    channel,
                    note,
                    flags,
                }),
            }
        }
        0b1011 => {
            // Control Change
            let control = bytes[2];
            let _reserved = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::ControlChange {
                    channel,
                    control,
                    data,
                }),
            }
        }
        0b0010 => {
            // Registered Controller (RPN)
            let bank = bytes[2];
            let index = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::RegsteredCtrl {
                    channel,
                    bank,
                    index,
                    data,
                }),
            }
        }
        0b0011 => {
            // Assignable Controller (NRPN)
            let bank = bytes[2];
            let index = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::AssignableCtrl {
                    channel,
                    bank,
                    index,
                    data,
                }),
            }
        }
        0b0100 => {
            // Relative Registered Controller (RPN)
            let bank = bytes[2];
            let index = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::RelRegCtrl {
                    channel,
                    bank,
                    index,
                    data,
                }),
            }
        }
        0b0101 => {
            // Relative Assignable Controller (NRPN)
            let bank = bytes[2];
            let index = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::RelAssnCtrl {
                    channel,
                    bank,
                    index,
                    data,
                }),
            }
        }
        0b1100 => {
            // Program change
            let _reserved0 = bytes[2];
            let options = bytes[3];
            let program = bytes[4];
            let _reserved1 = bytes[5];
            let bank = u16::from_be_bytes([bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::ProgramChange {
                    channel,
                    bank,
                    options,
                    program,
                }),
            }
        }
        0b1101 => {
            // Channel pressure
            let _reserved0 = bytes[2];
            let _reserved1 = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::ChannelPressure {
                    channel,
                    data,
                }),
            }
        }
        0b1110 => {
            // pitch bend
            let _reserved0 = bytes[2];
            let _reserved1 = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::PitchBend { channel, data }),
            }
        }
        0b0110 => {
            // per-note pitch bend
            let note = bytes[2];
            let _reserved = bytes[3];
            let data = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            MidiMessage {
                group,
                data: MidiMessageData::ChannelVoice(ChannelVoice::PerNotePitchBnd {
                    channel,
                    note,
                    data,
                }),
            }
        }
        _ => unreachable!(),
    }
}

fn utility(bytes: [u8; 4]) -> MidiMessage {
    let group = bytes[0] & 0x0f;
    let status = bytes[1] >> 4;
    let data = match status {
        0b0000 => MidiMessageData::Utility(Utility::NoOp),
        0b0001 => {
            MidiMessageData::Utility(Utility::JrClock(u16::from_le_bytes([bytes[2], bytes[3]])))
        }
        0b0010 => MidiMessageData::Utility(Utility::JrTimestamp(u16::from_le_bytes([
            bytes[2], bytes[3],
        ]))),
        _ => unreachable!(),
    };
    MidiMessage { group, data }
}

fn system(bytes: [u8; 4]) -> MidiMessage {
    let group = bytes[0] & 0x0f;
    let status = bytes[1];
    let msg = match status {
        0xf0 | 0xf4 | 0xf5 | 0xf7 | 0xf9 | 0xfd => MidiMessageData::Undefined(UMP::U32(bytes)), // reserved
        0xf1 => MidiMessageData::SystemCommon(SystemCommon::TimeCode(bytes[2])),
        0xf2 => {
            let lsb = bytes[2];
            let msb = bytes[3];
            let spp = ((msb as u16) << 7) | (lsb as u16);
            MidiMessageData::SystemCommon(SystemCommon::SongPositionPointer(spp))
        }
        0xf3 => MidiMessageData::SystemCommon(SystemCommon::SongSelect(bytes[2])),
        0xf6 => MidiMessageData::SystemCommon(SystemCommon::TuneRequest),
        0xf8 => MidiMessageData::SystemRealtime(SystemRealtime::TimingClock),
        0xfa => MidiMessageData::SystemRealtime(SystemRealtime::Start),
        0xfb => MidiMessageData::SystemRealtime(SystemRealtime::Continue),
        0xfc => MidiMessageData::SystemRealtime(SystemRealtime::Stop),
        0xfe => MidiMessageData::SystemRealtime(SystemRealtime::ActiveSensing),
        0xff => MidiMessageData::SystemRealtime(SystemRealtime::Reset),
        _ => unreachable!(),
    };
    MidiMessage { group, data: msg }
}

fn data64(bytes: [u8; 8]) -> MidiMessage {
    let group = bytes[0] & 0x0f;
    let status = bytes[1] >> 4;
    let data = match status {
        0 => Data64::SinglePacket(bytes),
        1 => Data64::Start(bytes),
        2 => Data64::Continue(bytes),
        3 => Data64::End(bytes),
        _ => unreachable!(),
    };
    MidiMessage {
        group,
        data: MidiMessageData::Data64(data),
    }
}

fn data128(bytes: [u8; 16]) -> MidiMessage {
    let group = bytes[0] & 0x0f;
    let status = bytes[1] >> 4;
    let data = match status {
        0 => Data128::SinglePacket(bytes),
        1 => Data128::Start(bytes),
        2 => Data128::Continue(bytes),
        3 => Data128::End(bytes),
        4 => Data128::MixedDataSetHeader(bytes),
        5 => Data128::MixedDataSetPayload(bytes),
        _ => unreachable!(),
    };
    MidiMessage {
        group,
        data: MidiMessageData::Data128(data),
    }
}

#[cfg(test)]
mod tests {
    use crate::message::*;
    use crate::ump::*;

    #[test]
    fn utility() {
        let jr_clk = Utility::JrClock(0xdead).into_msg(0);
        let jr_ts = Utility::JrTimestamp(0xdead).into_msg(1);
        let no_op = Utility::NoOp.into_msg(2);
        let jr_clk_ump: UMP = jr_clk.into();
        let jr_ts_ump: UMP = jr_ts.into();
        let no_op_ump: UMP = no_op.into();
        assert_eq!(jr_clk, jr_clk_ump.into());
        assert_eq!(jr_ts, jr_ts_ump.into());
        assert_eq!(no_op, no_op_ump.into());
    }

    #[test]
    fn channel_voice() {}

    #[test]
    fn system() {}
}
