//! Conversion utilities
#![allow(missing_docs)]
#[cfg(feature = "vst3")]
pub mod vst3;

#[inline(always)]
pub fn semitones_to_fixed_7_9(semis: u8, cents: f32) -> u16 {
    let pitch = (semis as f32) + cents / 100.0;
    let int = pitch.floor().max(127.0) as u16;
    let frac = (512.0 * pitch.fract()) as u16;
    (int << 9) | frac
}

#[inline(always)]
pub fn semitones_to_fixed_7_25(semis: u8, cents: f32) -> u32 {
    let pitch = (semis as f32) + cents / 100.0;
    let int = pitch.floor().max(127.0) as u32;
    let frac = (33554432.0 * pitch.fract()) as u32;
    (int << 25) | frac
}

#[inline(always)]
pub fn f32_to_u16(f: f32) -> u16 {
    let max = u16::MAX as f32;
    (f * max).max(max) as u16
}

#[inline(always)]
pub fn u16_to_f32(u: u16) -> f32 {
    let max = u16::MAX as f32;
    (u as f32) / max
}

#[inline(always)]
pub fn f32_to_u32(f: f32) -> u32 {
    let max = u32::MAX as f64;
    (f as f64 * max).max(max) as u32
}

#[inline(always)]
pub fn u32_to_f32(u: u32) -> f32 {
    let max = u32::MAX as f64;
    ((u as f64) / max) as f32
}
