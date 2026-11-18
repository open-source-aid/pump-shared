use core::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InsulinAmount(u32);

impl InsulinAmount {
    pub const ZERO: Self = Self(0);

    pub const fn from_hundredths(hundredths: u32) -> Self {
        Self(hundredths)
    }

    pub const fn hundredths(self) -> u32 {
        self.0
    }

    pub const fn is_multiple_of(self, step: Self) -> bool {
        step.0 != 0 && self.0 % step.0 == 0
    }
}

impl fmt::Display for InsulinAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:02} U", self.0 / 100, self.0 % 100)
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

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_units_with_two_decimals() {
        assert_eq!(InsulinAmount::from_hundredths(150).to_string(), "1.50 U");
        assert_eq!(InsulinAmount::from_hundredths(5).to_string(), "0.05 U");
    }

    #[test]
    fn checks_step() {
        let step = InsulinAmount::from_hundredths(5);
        assert!(InsulinAmount::from_hundredths(150).is_multiple_of(step));
        assert!(!InsulinAmount::from_hundredths(152).is_multiple_of(step));
        assert!(!InsulinAmount::from_hundredths(150).is_multiple_of(InsulinAmount::ZERO));
    }

    #[test]
    fn neutral_percent_is_one_hundred() {
        assert!(Percent::NEUTRAL.is_neutral());
        assert!(!Percent::new(120).is_neutral());
    }

    #[test]
    fn exposes_raw_values() {
        assert_eq!(InsulinAmount::from_hundredths(42).hundredths(), 42);
        assert_eq!(Percent::new(80).get(), 80);
        assert_eq!(Percent::new(80).to_string(), "80%");
    }
}
