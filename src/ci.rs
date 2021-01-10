//! MIDI Capability Inquiry defines system exclusive messages for discovering other devices
//! and managing their connections.
use crate::muid::MUID;

/// Metadata required for MIDI CI Discovery
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct DeviceDiscovery {
    /// The [MUID] of the device
    pub muid: MUID,
    /// A device manufacturer identifier
    pub manufacturer: [u8; 3],
    /// A device family identifier
    pub family: [u8; 2],
    /// A device model identifier
    pub model: [u8; 2],
    /// The software/firmware/hardware revision number
    pub revision: u32,
    /// Bitflags to notate CI support
    pub ci_support: u8,
    /// Maximum size of a sysex message the device can receive. At least 128 bytes.
    pub max_sysex_size: u32,
}

/// Used to handle MUID collisions.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct InvalidateMUID {
    pub target: MUID,
}

/// Default response when receiving a message not understood
///
/// Used if:
/// - Received a CI message the device does not understand
/// - Received a CI message with unsupported MIDI CI version
/// - Received a malformed CI message
/// - Received a profile enable/disable message for a profile that the device does not support
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct NotAcknowledged;

/// Single byte representing the MIDI version used by a protocol
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MidiVersion {
    Midi1 = 0x01,
    Midi2 = 0x02,
}

/// 3 bytes identifying a protocol used in protocol negotation
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Protocol {
    /// The MIDI version of the protocol to use
    pub midi_version: MidiVersion,
    /// The version of the protocol to use
    pub version: u8,
    /// Any extensions of the protocol to use
    pub extensions: u8,
}

impl Protocol {
    /// Default midi1 protocol
    pub fn midi1() -> Self {
        Self::new(MidiVersion::Midi1)
    }
    /// Default midi2 protocol
    pub fn midi2() -> Self {
        Self::default()
    }
    /// Create a new protocol
    pub fn new(mv: MidiVersion) -> Self {
        Self {
            midi_version: mv,
            version: 0,
            extensions: 0,
        }
    }
    /// Notate the device supports the jitter reduction extension
    pub fn with_jitter_reduction(mut self) -> Self {
        self.extensions |= 0b0000_00001;
        self
    }
    /// Notate the device supports UMPs larger than 64 bits. Only applicable for MIDI 1.
    pub fn with_large_packets (mut self) -> Self {
        debug_assert_eq!(self.midi_version, MidiVersion::Midi1);
        self.extensions |= 0b0000_0010;
        self
    }
    /// Add an additional version number. NOTE: is not MIDI 1 vs MIDI 2.
    pub fn with_additional_version(mut self, vers: u8) -> Self {
        self.version = vers;
        self
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Self::new(MidiVersion::Midi2)
    }
}

/// Message to begin initializing protocol negotation. The spec allows for multiple protocls
/// to be supported, this crate only supports a single preferred protocol until const generics
/// are stabilized, as Rust does not support VLA members in structs.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct InitiateProtocolNegotiation {
    authority: u8,
    preferred: Protocol,
}

impl InitiateProtocolNegotiation {
    /// Construct a new protocol negotation message
    pub fn new(auth: AuthorityLevel) -> Self {
        Self {
            authority: auth as u8,
            preferred: Protocol::default(),
        }
    }
    /// annotate additional authority level. Must be less than 16
    pub fn with_additional_authority(mut self, a: u8) -> Self {
        debug_assert!(a < 16);
        self.authority |= a;
        self
    }
    /// annotate a different preferred protocol
    pub fn with_preferred_protocol(mut self, p: Protocol) -> Self {
        self.preferred = p;
        self
    }
}

impl DeviceDiscovery {
    /// Construct a new device with a MUID. This should be sufficiently
    /// unique and does not need to be persistent.
    pub fn new(muid: MUID) -> Self {
        Self {
            muid,
            manufacturer: [0; 3],
            family: [0; 2],
            model: [0; 2],
            revision: 0,
            ci_support: 0,
            max_sysex_size: 128,
        }
    }
    /// Add a unique manufacturer code.
    pub fn with_manufacturer_code(mut self, manu: [u8; 3]) -> Self {
        self.manufacturer = manu;
        self
    }
    /// Add a device family code
    pub fn with_family_code(mut self, family: [u8; 2]) -> Self {
        self.family = family;
        self
    }
    /// Add a device model code
    pub fn with_model_code(mut self, model: [u8; 2]) -> Self {
        self.model = model;
        self
    }
    /// Add a software/firmware/hardware revision number
    pub fn with_revision(mut self, revision: u32) -> Self {
        self.revision = revision;
        self
    }
    /// Define the maximum length of sysex message the device may
    /// receive. Must be at least 128.
    pub fn with_max_sysex_length(mut self, len: u32) -> Self {
        debug_assert!(len >= 128);
        self.max_sysex_size = len;
        self
    }
    /// Notate the device supports protocol negotiation
    pub fn with_protocol_negotiation(mut self) -> Self {
        self.ci_support |= 0b0000_0010;
        self
    }
    /// Notate the device supports profile configuration.
    pub fn with_profile_configuration(mut self) -> Self {
        self.ci_support |= 0b0000_0100;
        self
    }
    /// Notate the device supports property exchange.
    pub fn with_property_exchange(mut self) -> Self {
        self.ci_support |= 0b0000_1000;
        self
    }
}

/// Sent when a device requests a new protocol for communication
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct SetNewProtocol {
    pub protocol:Protocol,
    pub authority:u8
}

impl SetNewProtocol {
    /// Construct a new SetNewProtocol message
    pub fn new(p:Protocol, a:AuthorityLevel) -> Self {
        Self {
            protocol: p, 
            authority: a as u8
        }
    }
    /// Notate additional authority for the request. Must be less than 16
    pub fn with_additional_authority (mut self, a:u8) -> Self {
        debug_assert!(a < 16); 
        self.authority |= a;
        self
    }
}

pub trait CapabilityInquiryMessage {
    /// The authority level of the message
    fn authority(&self) -> AuthorityLevel {
        AuthorityLevel::ReservedLower
    }

    /// The category of a CI message
    fn category(&self) -> Category;

    /// The subcategory (type) of CI message
    fn subcategory(&self) -> u8;

    /// Source of the message
    fn source(&self) -> MUID;

    /// Destination address of the message
    fn dest(&self) -> MUID;

    /// Used for targeting a specific MIDI channel on a device, value 0x7f corresponds
    /// to the entire MIDI port.
    fn device_id(&self) -> u8 {
        0x7f
    }
    fn data(&self) -> &'_ [u8];
}

/// Represents the various management authority levels required by some MIDI specifications.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AuthorityLevel {
    /// Reserved for future use.
    ReservedUpper = 0x70,
    /// Owner of many devices, eg a PC or router
    NodeServer = 0x60,
    Gateway = 0x50,
    Translator = 0x40,
    Endpoint = 0x30,
    /// Processors like arpeggiators, sequencers, etc
    EventProcessor = 0x20,
    Transport = 0x10,
    /// Reserved for future use
    ReservedLower = 0x00,
}
