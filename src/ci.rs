//! MIDI Capability Inquiry defines system exclusive messages for discovering other devices 
//! and managing their connections.

pub enum CiMessage {
    ProtocolNegotiation,
    ProfileConfiguration,
    PropertyExchange,
    Management,
}

pub struct InitProtocolNegotiation {
    source:u32,
    dest:u32,
    authority:u8,
    num_protocols:u8,
    preferred:[u8;5],
    other:[[u8;5];1],
}
