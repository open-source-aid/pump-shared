use alloc::vec::Vec;
use core::fmt;
use core::time::Duration;

use crate::BasalRate;

pub const DAY: Duration = Duration::from_secs(24 * 3600);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasalSegment {
    pub start: Duration,
    pub rate: BasalRate,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasalProfile(Vec<BasalSegment>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileError {
    Empty,
    NotFromMidnight,
    NotIncreasing,
    PastMidnight,
}

impl BasalProfile {
    pub fn new(segments: Vec<BasalSegment>) -> Result<Self, ProfileError> {
        let first = segments.first().ok_or(ProfileError::Empty)?;
        if !first.start.is_zero() {
            return Err(ProfileError::NotFromMidnight);
        }
        if segments.windows(2).any(|w| w[0].start >= w[1].start) {
            return Err(ProfileError::NotIncreasing);
        }
        if segments.iter().any(|s| s.start >= DAY) {
            return Err(ProfileError::PastMidnight);
        }
        Ok(Self(segments))
    }

    pub fn segments(&self) -> &[BasalSegment] {
        &self.0
    }

    pub fn rate_at(&self, since_midnight: Duration) -> BasalRate {
        let offset = Duration::from_secs(since_midnight.as_secs() % DAY.as_secs());
        self.0
            .iter()
            .take_while(|s| s.start <= offset)
            .last()
            .map_or(BasalRate::ZERO, |s| s.rate)
    }
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "basal profile has no segment",
            Self::NotFromMidnight => "basal profile must start at midnight",
            Self::NotIncreasing => "basal segments must start in increasing order",
            Self::PastMidnight => "basal segment starts after the end of the day",
        })
    }
}

impl core::error::Error for ProfileError {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn segment(hour: u64, milli: u32) -> BasalSegment {
        BasalSegment {
            start: Duration::from_secs(hour * 3600),
            rate: BasalRate::from_milliunits_per_hour(milli),
        }
    }

    #[test]
    fn finds_the_rate_at_any_time_of_day() {
        let profile =
            BasalProfile::new(vec![segment(0, 800), segment(6, 1_200), segment(22, 700)]).unwrap();
        assert_eq!(profile.segments().len(), 3);
        let at = |hours: u64| {
            profile
                .rate_at(Duration::from_secs(hours * 3600))
                .milliunits_per_hour()
        };
        assert_eq!(at(0), 800);
        assert_eq!(at(5), 800);
        assert_eq!(at(6), 1_200);
        assert_eq!(at(23), 700);
        assert_eq!(at(24 + 7), 1_200);
    }

    #[test]
    fn rejects_malformed_profiles() {
        assert_eq!(BasalProfile::new(vec![]), Err(ProfileError::Empty));
        assert_eq!(
            BasalProfile::new(vec![segment(1, 800)]),
            Err(ProfileError::NotFromMidnight)
        );
        assert_eq!(
            BasalProfile::new(vec![segment(0, 800), segment(0, 900)]),
            Err(ProfileError::NotIncreasing)
        );
        assert_eq!(
            BasalProfile::new(vec![segment(0, 800), segment(24, 900)]),
            Err(ProfileError::PastMidnight)
        );
    }

    #[test]
    fn errors_have_messages() {
        for error in [
            ProfileError::Empty,
            ProfileError::NotFromMidnight,
            ProfileError::NotIncreasing,
            ProfileError::PastMidnight,
        ] {
            assert!(!error.to_string().is_empty());
        }
    }
}
