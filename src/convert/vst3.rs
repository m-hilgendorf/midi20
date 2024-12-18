//! Convert to/from VST3 event types.

use crate::{
    convert,
    message::{
        channel2::{self, ChannelVoice},
        Data,
    },
};
use core::convert::TryFrom;

/// Conversion errors.
pub enum Error {
    /// An unknown event happened
    UnknownEventType(u16),
}

impl TryFrom<&vst3::Steinberg::Vst::Event> for Data {
    type Error = Error;
    fn try_from(value: &vst3::Steinberg::Vst::Event) -> Result<Self, Self::Error> {
        match value.r#type as u32 {
            vst3::Steinberg::Vst::Event_::EventTypes_::kChordEvent => {
                let _event = unsafe { &value.__field0.chord };
                todo!()
            }
            vst3::Steinberg::Vst::Event_::EventTypes_::kDataEvent => {
                let _event = unsafe { &value.__field0.data };
                todo!()
            }
            vst3::Steinberg::Vst::Event_::EventTypes_::kLegacyMIDICCOutEvent => {
                let _event = unsafe { &value.__field0.midiCCOut };
                todo!()
            }
            vst3::Steinberg::Vst::Event_::EventTypes_::kNoteExpressionTextEvent => {
                let _event = unsafe { &value.__field0.noteExpressionText };
                todo!()
            }
            vst3::Steinberg::Vst::Event_::EventTypes_::kNoteExpressionValueEvent => {
                let _event = unsafe { &value.__field0.noteExpressionValue };
                todo!()
            }
            vst3::Steinberg::Vst::Event_::EventTypes_::kNoteOffEvent => {
                let event = unsafe { &value.__field0.noteOff };
                let attribute = if event.tuning != 0.0 {
                    let tuning = convert::semitones_to_fixed_7_9(event.pitch as u8, event.tuning);
                    Some(channel2::Attribute::Pitch79(tuning))
                } else {
                    None
                };
                let data = ChannelVoice::note_off(
                    event.pitch as u8,
                    convert::f32_to_u16(event.velocity),
                    attribute,
                )
                .with_channel(event.channel as u8);
                Ok(Data::ChannelVoice(data))
            }
            vst3::Steinberg::Vst::Event_::EventTypes_::kNoteOnEvent => {
                let event = unsafe { &value.__field0.noteOn };
                let attribute = if event.tuning != 0.0 {
                    let tuning = convert::semitones_to_fixed_7_9(event.pitch as u8, event.tuning);
                    Some(channel2::Attribute::Pitch79(tuning))
                } else {
                    None
                };
                let data = ChannelVoice::note_on(
                    event.pitch as u8,
                    convert::f32_to_u16(event.velocity),
                    attribute,
                )
                .with_channel(event.channel as u8);
                Ok(Data::ChannelVoice(data))
            }
            vst3::Steinberg::Vst::Event_::EventTypes_::kPolyPressureEvent => {
                let event = unsafe { &value.__field0.polyPressure };
                let data = ChannelVoice::poly_pressure(
                    event.pitch as u8,
                    convert::f32_to_u32(event.pressure),
                )
                .with_channel(event.channel as u8);
                Ok(Data::ChannelVoice(data))
            }
            vst3::Steinberg::Vst::Event_::EventTypes_::kScaleEvent => {
                let _event = unsafe { &value.__field0.scale };
                todo!()
            }
            r#type => return Err(Error::UnknownEventType(r#type as u16)),
        }
    }
}
