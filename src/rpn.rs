//! Registered per-note controls.

/// RPN Indices defined in MIDI specification.
/// Only pertains to RPN Bank 0.
pub enum RegisteredCtlStatus {
    /// 0x0000. Sets pitch bend range in HCUs and cents
    PitchBendRange {
        /// The pitch bend range in semitones.
        semitones: u8,

        /// The pitch bend range in cents.
        cents: u8,
    },
    /// 0x0002. Set the tuning
    CoarseTuning(u8),

    /// 0x0003. Selects a tuning program
    TuningProgramChange(u8),

    /// 0x0004. Selects a tuning bank
    TuningBankSelect(u8),

    /// 0x0006. Declares the number of Channels used for an MPE Lower or Upper Zone
    MPEConfiguration(u8),

    /// 0x0007. Sets per-note pitch bend range in HCUs and cents
    PerNotePitchBendRange(u32),

    /// Any other RPN control status.
    Other(u32),
}

#[allow(missing_docs)]
pub enum RegisteredPerNoteCtlStatus {
    Modulation(u32),
    Breath(u32),
    Pitch7_25(u32),
    Volume(u32),
    Balance(u32),
    Pan(u32),
    Expression(u32),

    /// defaults to Sound Variation
    SoundController1(u32),

    /// defaults to Timbre/Harmonic intensity
    SoundController2(u32),

    /// defaults to Release Time
    SoundController3(u32),

    /// defaults to Attack Time
    SoundController4(u32),

    /// defaults to Brightness
    SoundController5(u32),

    /// defaults to Decay Time
    SoundController6(u32),

    /// defaults to Vibrato Rate
    SoundController7(u32),

    /// defaults to Vibrato Depth
    SoundController8(u32),

    /// defaults to Vibrato Delay
    SoundController9(u32),
    SoundController10(u32),

    /// defaults to Reverb Send Level
    FX1Depth(u32),

    /// formerly Tremolo depth
    FX2Depth(u32),

    /// defaults to Chorus Send Level
    FX3Depth(u32),

    /// formerly Celeste [Detune] depth
    FX4Depth(u32),

    /// formerly Phaser depth
    FX5Depth(u32),

    Other {
        /// The RPN bank value.
        bank: u8,

        /// The RPN index value.
        index: u8,

        /// The RPN data.
        data: u32,
    },
}

impl From<(u8, u8, u32)> for RegisteredPerNoteCtlStatus {
    fn from(value: (u8, u8, u32)) -> Self {
        let data = value.2;
        match value.0 {
            1 => Self::Modulation(data),
            2 => Self::Breath(data),
            3 => Self::Pitch7_25(data),
            7 => Self::Volume(data),
            8 => Self::Balance(data),
            10 => Self::Pan(data),
            11 => Self::Expression(data),
            70 => Self::SoundController1(data),
            71 => Self::SoundController2(data),
            72 => Self::SoundController3(data),
            73 => Self::SoundController4(data),
            74 => Self::SoundController5(data),
            75 => Self::SoundController6(data),
            76 => Self::SoundController7(data),
            77 => Self::SoundController8(data),
            78 => Self::SoundController9(data),
            79 => Self::SoundController10(data),
            91 => Self::FX1Depth(data),
            92 => Self::FX2Depth(data),
            93 => Self::FX3Depth(data),
            94 => Self::FX4Depth(data),
            95 => Self::FX5Depth(data),
            _ => Self::Other {
                bank: value.0,
                index: value.1,
                data,
            },
        }
    }
}
