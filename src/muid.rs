//! Helpers for generating and managing MUIDs, unique 32 bit identifiers for a device. Only
//! available if the `no-std` feature is not specified. Otherwise one should fallback to
//! the recommendations of the specification.

/// Represents braodcast messages, those intended to reach all devices
pub const BROADCAST: MUID = MUID(0x0fff_ffff);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct MUID(u32);

impl MUID {
    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

/// Generates a MUID by hashing the number of milliseconds since UNIX_EPOCH, truncated
/// to 32 bits. This method takes _at least_ one millisecond to complete by calling
/// thread::sleep, which is intended to provide unique IDs in the case that many
/// are generated in a loop.
#[cfg(not(feature = "no-std"))]
pub fn new_muid() -> MUID {
    use std::thread;
    use std::time;
    thread::sleep(time::Duration::from_millis(1));
    let mut id = BROADCAST.0;
    while id >= 0x0fff_fff0 {
        let now = (time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            & 0xffff_ffff) as u32;
        id = ((now >> 16) ^ now).wrapping_mul(0x119d_e1f3);
        id = ((id >> 16) ^ id).wrapping_mul(0x119d_e1f3);
        id = id ^ (id >> 16);
    }
    MUID(id)
}
