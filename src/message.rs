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
