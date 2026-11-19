use core::fmt;
use core::time::Duration;

use crate::{BasalRate, InsulinAmount, Report};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Outcome {
    Completed(Report),
    Approximated(Approximation),
    Refused(Refusal),
    Unknown,
    Failed(Failure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Approximation {
    Bolus {
        requested: InsulinAmount,
        delivered: InsulinAmount,
        duration: Duration,
    },
    Basal {
        requested: BasalRate,
        achieved: BasalRate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Refusal {
    NotBuilt,
    NotPaired,
    NotRunning,
    NotSuspended,
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
            Self::NotSuspended => "pump is not suspended",
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
            Refusal::NotSuspended,
            Refusal::OutOfLimits,
            Refusal::Busy,
            Refusal::Unsupported,
        ];
        for refusal in refusals {
            assert!(!refusal.to_string().is_empty());
        }
    }

    #[test]
    fn approximation_separates_bolus_and_basal() {
        let bolus = Approximation::Bolus {
            requested: InsulinAmount::from_milliunits(2_000),
            delivered: InsulinAmount::from_milliunits(1_850),
            duration: Duration::from_secs(1_800),
        };
        let basal = Approximation::Basal {
            requested: BasalRate::from_milliunits_per_hour(1_500),
            achieved: BasalRate::from_milliunits_per_hour(1_480),
        };
        assert_eq!(bolus, bolus);
        assert_eq!(basal, basal);
        assert_ne!(bolus, basal);
        assert_ne!(
            bolus,
            Approximation::Bolus {
                requested: InsulinAmount::from_milliunits(2_000),
                delivered: InsulinAmount::from_milliunits(2_000),
                duration: Duration::from_secs(1_800),
            }
        );
        assert_ne!(
            basal,
            Approximation::Basal {
                requested: BasalRate::from_milliunits_per_hour(1_500),
                achieved: BasalRate::from_milliunits_per_hour(1_500),
            }
        );
        assert_ne!(Outcome::Approximated(bolus), Outcome::Approximated(basal));
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
