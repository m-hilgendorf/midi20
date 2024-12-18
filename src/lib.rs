#![deny(missing_docs)]
//! Types and helpers for building MIDI-2 capable software in Rust
#![cfg_attr(feature = "no-std", no_std)]
pub mod ci;
pub mod convert;
pub mod message;
pub mod muid;
pub mod packet;
pub mod rpn;
