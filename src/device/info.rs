use alloc::string::String;
use core::fmt;

use crate::Limits;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address([u8; 6]);

impl Address {
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub const fn to_bytes(self) -> [u8; 6] {
        self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d, e, g] = self.0;
        write!(f, "{a:02X}:{b:02X}:{c:02X}:{d:02X}:{e:02X}:{g:02X}")
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sequence(u32);

impl Sequence {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EngineState {
    Loading,
    Unpaired,
    Pairing,
    Paired,
    KeyRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceInfo {
    pub serial: String,
    pub firmware: String,
    pub variant: &'static str,
    pub limits: Limits,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_address_as_colon_hex() {
        let address = Address::new([0xEC, 0x2A, 0xF0, 0x01, 0x02, 0x0A]);
        assert_eq!(address.to_string(), "EC:2A:F0:01:02:0A");
    }

    #[test]
    fn exposes_raw_values() {
        assert_eq!(
            Address::new([1, 2, 3, 4, 5, 6]).to_bytes(),
            [1, 2, 3, 4, 5, 6]
        );
        assert_eq!(Sequence::new(7).get(), 7);
    }
}
