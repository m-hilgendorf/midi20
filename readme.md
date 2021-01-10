# `midi2`
MIDI 2 is the next generation of MIDI, allowing bidirectional communication between enabled devices. This crate contains types and helpers for building MIDI-2 capable software in Rust. Specifications can be found at [midi.org](https://midi.org).

### Features: 
- [x] `#![no_std]`
- [x] serialize/deserialize from universal midi packets (UMP) 
- [x] MUID generation (requires `std`)
- [x] MIDI 2 AST 

### Todos: 
- [ ] System exclusive helpers
- [ ] Capability inquiry (MIDI-CI)
- [ ] Property exchange (MIDI-PE)
- [ ] Conversion from MIDI 1.0 channel voice messages to MIDI 2.0 (increase resolution)
