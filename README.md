[![Discord Chat][discord-img]][discord-url]

# `midi2`
MIDI 2 is the next generation of MIDI, allowing bidirectional communication between enabled devices. This crate contains types and helpers for building MIDI-2 capable software in Rust. Specifications can be found at [midi.org](https://midi.org).

MIDI 2.0 is very much in an "alpha" state in the industry - operating systems have just begun to offer initial support for MIDI 2.0, and at the time of writing (Sep 2023) there are few (if any) devices that are available to send or receive MIDI 2 packets. 

The goal of this crate is to allow device and application authors to encode/decode and interpret MIDI 2.0 messages in idiomatic Rust, with little overhead introduced by the underlying data representation itself.

### Features: 
- [x] `#![no_std]`
- [x] serialize/deserialize from universal midi packets (UMP) 
- [x] MUID generation (requires `std`)
- [x] MIDI 2 AST 
- [x] Conversion from MIDI 1.0 channel voice messages to MIDI 2.0 (increase resolution)

### Todos: 
- [ ] System exclusive helpers
- [ ] Capability inquiry (MIDI-CI)
- [ ] Property exchange (MIDI-PE)

[discord-url]: https://discord.com/channels/590254806208217089/932384555900493945
[discord-img]: https://img.shields.io/discord/590254806208217089.svg?label=Discord&logo=discord&color=blue

### Help/Contributing

Contributions are very much welcome! Here are some good first issues:

- MIDI CI support.
- Property exchange.
- SMFCLIP2 file encoding/decoding
- Platform API conversion helpers:
  - [ ] [MacOS/iOS](https://developer.apple.com/documentation/coremidi/midiuniversalmessage)
  - [ ] [Android](https://source.android.com/docs/core/audio/midi)
  - [ ] [Windows](https://github.com/microsoft/midi)
  - [ ] [Linux/ALSA](https://github.com/alsa-project/alsa-utils)

If you make a PR, make sure to update the `authors` field in [Cargo.toml](./Cargo.toml). 

Come discuss on the [Rust Audio discord!](https://discord.com/channels/590254806208217089/932384555900493945)
