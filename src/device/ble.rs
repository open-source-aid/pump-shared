use alloc::vec::Vec;
use core::fmt;

use crate::Address;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uuid([u8; 16]);

impl Uuid {
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().enumerate().try_for_each(|(i, byte)| {
            let dash = if matches!(i, 4 | 6 | 8 | 10) { "-" } else { "" };
            write!(f, "{dash}{byte:02x}")
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BleOp {
    Connect {
        address: Address,
    },
    Disconnect,
    Read {
        service: Uuid,
        characteristic: Uuid,
    },
    Write {
        service: Uuid,
        characteristic: Uuid,
        data: Vec<u8>,
    },
    Subscribe {
        service: Uuid,
        characteristic: Uuid,
    },
    RequestFastInterval,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BleResult {
    Done,
    Data(Vec<u8>),
    Status(u8),
    Timeout,
    Disconnected,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_uuid_hyphenated() {
        let uuid = Uuid::from_bytes([
            0x66, 0x9a, 0x0c, 0x20, 0x00, 0x08, 0x96, 0x9e, 0xe2, 0x11, 0x9e, 0xb1, 0xe0, 0xf2,
            0x73, 0xd9,
        ]);
        assert_eq!(uuid.to_string(), "669a0c20-0008-969e-e211-9eb1e0f273d9");
    }

    #[test]
    fn exposes_bytes() {
        assert_eq!(Uuid::from_bytes([3; 16]).as_bytes(), &[3; 16]);
    }
}
