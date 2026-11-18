use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use zeroize::Zeroizing;

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(Zeroizing<Vec<u8>>);

impl Secret {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }

    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret({} bytes)", self.0.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyRef(String);

impl KeyRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::vec;

    #[test]
    fn debug_never_shows_bytes() {
        let secret = Secret::new(vec![0xAB; 32]);
        assert_eq!(format!("{secret:?}"), "Secret(32 bytes)");
    }

    #[test]
    fn exposes_contents() {
        assert_eq!(Secret::new(vec![1, 2]).expose(), &[1, 2]);
        assert_eq!(KeyRef::new("server").as_str(), "server");
    }
}
