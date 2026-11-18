use alloc::vec::Vec;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Outcome {
    Completed { detail: Vec<u8> },
    Refused(Refusal),
    Unknown,
    Failed(Failure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Refusal {
    NotBuilt,
    NotPaired,
    NotRunning,
    OutOfLimits,
    Busy,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Failure {
    KeyRejected,
    Status(u8),
    Host,
    Disconnected,
    Protocol,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotBuilt => "not built into this driver",
            Self::NotPaired => "device not paired",
            Self::NotRunning => "pump is not running",
            Self::OutOfLimits => "outside the device limits",
            Self::Busy => "another operation is running",
            Self::Unsupported => "not supported by this device",
        })
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyRejected => f.write_str("key rejected by the device"),
            Self::Status(code) => write!(f, "device status {code:#04x}"),
            Self::Host => f.write_str("host action failed"),
            Self::Disconnected => f.write_str("link lost"),
            Self::Protocol => f.write_str("protocol error"),
        }
    }
}

impl core::error::Error for Refusal {}
impl core::error::Error for Failure {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn every_refusal_has_a_message() {
        let refusals = [
            Refusal::NotBuilt,
            Refusal::NotPaired,
            Refusal::NotRunning,
            Refusal::OutOfLimits,
            Refusal::Busy,
            Refusal::Unsupported,
        ];
        for refusal in refusals {
            assert!(!refusal.to_string().is_empty());
        }
    }

    #[test]
    fn every_failure_has_a_message() {
        let failures = [
            Failure::KeyRejected,
            Failure::Host,
            Failure::Disconnected,
            Failure::Protocol,
        ];
        for failure in failures {
            assert!(!failure.to_string().is_empty());
        }
        assert_eq!(Failure::Status(0x8B).to_string(), "device status 0x8b");
    }
}
