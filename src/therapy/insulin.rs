use core::fmt;
use core::time::Duration;

pub trait Quantity: Copy + Ord {
    fn raw(self) -> u64;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InsulinAmount(u32);

impl InsulinAmount {
    pub const ZERO: Self = Self(0);

    pub const fn from_milliunits(milliunits: u32) -> Self {
        Self(milliunits)
    }

    pub const fn milliunits(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BasalRate(u32);

impl BasalRate {
    pub const ZERO: Self = Self(0);

    pub const fn from_milliunits_per_hour(milliunits: u32) -> Self {
        Self(milliunits)
    }

    pub const fn milliunits_per_hour(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(u16);

impl Percent {
    pub const NEUTRAL: Self = Self(100);

    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }

    pub const fn is_neutral(self) -> bool {
        self.0 == 100
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TempBasalRate {
    Percent(Percent),
    Absolute(BasalRate),
}

impl TempBasalRate {
    pub const fn is_neutral(self) -> bool {
        matches!(self, Self::Percent(p) if p.is_neutral())
    }
}

impl Quantity for InsulinAmount {
    fn raw(self) -> u64 {
        self.0.into()
    }
}

impl Quantity for BasalRate {
    fn raw(self) -> u64 {
        self.0.into()
    }
}

impl Quantity for Percent {
    fn raw(self) -> u64 {
        self.0.into()
    }
}

impl Quantity for Duration {
    fn raw(self) -> u64 {
        self.as_secs()
    }
}

fn write_milli(f: &mut fmt::Formatter<'_>, milli: u32, unit: &str) -> fmt::Result {
    write!(f, "{}.{:03} {unit}", milli / 1000, milli % 1000)
}

impl fmt::Display for InsulinAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_milli(f, self.0, "U")
    }
}

impl fmt::Display for BasalRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_milli(f, self.0, "U/h")
    }
}

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl fmt::Display for TempBasalRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Percent(p) => p.fmt(f),
            Self::Absolute(r) => r.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn displays_milliunits() {
        assert_eq!(InsulinAmount::from_milliunits(1_525).to_string(), "1.525 U");
        assert_eq!(InsulinAmount::from_milliunits(25).to_string(), "0.025 U");
        assert_eq!(
            BasalRate::from_milliunits_per_hour(850).to_string(),
            "0.850 U/h"
        );
    }

    #[test]
    fn exposes_raw_values() {
        assert_eq!(InsulinAmount::from_milliunits(42).milliunits(), 42);
        assert_eq!(
            BasalRate::from_milliunits_per_hour(7).milliunits_per_hour(),
            7
        );
        assert_eq!(Percent::new(80).get(), 80);
        assert_eq!(InsulinAmount::ZERO.raw(), 0);
        assert_eq!(BasalRate::ZERO.raw(), 0);
        assert_eq!(Percent::new(80).raw(), 80);
        assert_eq!(Duration::from_secs(90).raw(), 90);
    }

    #[test]
    fn only_a_hundred_percent_is_neutral() {
        assert!(TempBasalRate::Percent(Percent::NEUTRAL).is_neutral());
        assert!(!TempBasalRate::Percent(Percent::new(120)).is_neutral());
        assert!(!TempBasalRate::Absolute(BasalRate::ZERO).is_neutral());
    }

    #[test]
    fn displays_temp_basal_rates() {
        assert_eq!(TempBasalRate::Percent(Percent::new(80)).to_string(), "80%");
        let absolute = TempBasalRate::Absolute(BasalRate::from_milliunits_per_hour(1_200));
        assert_eq!(absolute.to_string(), "1.200 U/h");
    }
}
