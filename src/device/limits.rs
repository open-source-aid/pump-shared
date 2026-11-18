use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;

use crate::{BasalRate, InsulinAmount, Percent, Quantity, TempBasalRate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tier<T> {
    pub from: T,
    pub step: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bounds<T> {
    pub min: T,
    pub max: T,
    pub tiers: Vec<Tier<T>>,
}

impl<T: Quantity> Bounds<T> {
    pub fn uniform(min: T, max: T, step: T) -> Self {
        Self {
            min,
            max,
            tiers: vec![Tier { from: min, step }],
        }
    }

    fn step_at(&self, raw: u64) -> Option<u64> {
        self.tiers
            .iter()
            .rev()
            .find(|tier| tier.from.raw() <= raw)
            .map(|tier| tier.step.raw())
            .filter(|step| *step != 0)
    }

    pub fn allows(&self, value: T) -> bool {
        value >= self.min
            && value <= self.max
            && self
                .step_at(value.raw())
                .is_some_and(|step| value.raw() % step == 0)
    }

    pub fn floor(&self, value: T) -> Option<T> {
        let raw = value.min(self.max).raw();
        let step = self.step_at(raw)?;
        let floored = T::from_raw(raw - raw % step);
        self.allows(floored).then_some(floored)
    }

    pub fn nearest(&self, value: T) -> Option<T> {
        let down = self.floor(value);
        let up = self.values().into_iter().find(|v| *v >= value);
        match (down, up) {
            (Some(d), Some(u)) if u.raw() - value.raw() < value.raw() - d.raw() => Some(u),
            (Some(d), _) => Some(d),
            (None, up) => up,
        }
    }

    pub fn values(&self) -> Vec<T> {
        let mut values = Vec::new();
        let mut raw = self.min.raw();
        while raw <= self.max.raw() {
            let Some(step) = self.step_at(raw) else {
                break;
            };
            if raw % step == 0 {
                values.push(T::from_raw(raw));
                raw += step;
            } else {
                raw += step - raw % step;
            }
        }
        values
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Capabilities {
    pub reservoir_capacity: Option<InsulinAmount>,
    pub max_profile_segments: u8,
    pub min_segment_duration: Duration,
    pub history: bool,
    pub daily_totals: bool,
    pub patch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Limits {
    pub bolus: Bounds<InsulinAmount>,
    pub basal: Bounds<BasalRate>,
    pub temp_basal_percent: Option<Bounds<Percent>>,
    pub temp_basal_absolute: Option<Bounds<BasalRate>>,
    pub temp_basal_duration: Bounds<Duration>,
    pub extended_bolus_duration: Option<Bounds<Duration>>,
}

impl Limits {
    pub fn allows_temp_basal(&self, rate: TempBasalRate, duration: Duration) -> bool {
        let rate_ok = match rate {
            TempBasalRate::Percent(p) => self
                .temp_basal_percent
                .as_ref()
                .is_some_and(|b| b.allows(p)),
            TempBasalRate::Absolute(r) => self
                .temp_basal_absolute
                .as_ref()
                .is_some_and(|b| b.allows(r)),
        };
        rate_ok && self.temp_basal_duration.allows(duration)
    }

    pub fn allows_extended_bolus(&self, amount: InsulinAmount, duration: Duration) -> bool {
        self.bolus.allows(amount)
            && self
                .extended_bolus_duration
                .as_ref()
                .is_some_and(|b| b.allows(duration))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HALF_HOUR: Duration = Duration::from_secs(1800);

    fn mu(value: u32) -> InsulinAmount {
        InsulinAmount::from_milliunits(value)
    }

    fn rate(value: u32) -> BasalRate {
        BasalRate::from_milliunits_per_hour(value)
    }

    fn ypso_basal() -> Bounds<BasalRate> {
        Bounds {
            min: rate(20),
            max: rate(2_500),
            tiers: vec![
                Tier {
                    from: rate(0),
                    step: rate(10),
                },
                Tier {
                    from: rate(1_000),
                    step: rate(20),
                },
                Tier {
                    from: rate(2_000),
                    step: rate(100),
                },
            ],
        }
    }

    fn limits() -> Limits {
        Limits {
            bolus: Bounds::uniform(mu(100), mu(25_000), mu(100)),
            basal: ypso_basal(),
            temp_basal_percent: Some(Bounds::uniform(
                Percent::new(0),
                Percent::new(200),
                Percent::new(10),
            )),
            temp_basal_absolute: None,
            temp_basal_duration: Bounds::uniform(
                HALF_HOUR,
                Duration::from_secs(24 * 3600),
                HALF_HOUR,
            ),
            extended_bolus_duration: None,
        }
    }

    #[test]
    fn checks_range_and_tiered_step() {
        let basal = ypso_basal();
        assert!(basal.allows(rate(990)));
        assert!(basal.allows(rate(1_020)));
        assert!(!basal.allows(rate(1_010)));
        assert!(basal.allows(rate(2_100)));
        assert!(!basal.allows(rate(2_120)));
        assert!(!basal.allows(rate(10)));
        assert!(!basal.allows(rate(2_600)));
    }

    #[test]
    fn floors_to_the_grid() {
        let basal = ypso_basal();
        assert_eq!(basal.floor(rate(1_015)), Some(rate(1_000)));
        assert_eq!(basal.floor(rate(2_150)), Some(rate(2_100)));
        assert_eq!(basal.floor(rate(9_999)), Some(rate(2_500)));
        assert_eq!(basal.floor(rate(15)), None);
    }

    #[test]
    fn rounds_to_the_nearest_value_preferring_down_on_ties() {
        let basal = ypso_basal();
        assert_eq!(basal.nearest(rate(1_015)), Some(rate(1_020)));
        assert_eq!(basal.nearest(rate(1_010)), Some(rate(1_000)));
        assert_eq!(basal.nearest(rate(1_003)), Some(rate(1_000)));
        assert_eq!(basal.nearest(rate(5)), Some(rate(20)));
        assert_eq!(basal.nearest(rate(9_999)), Some(rate(2_500)));
    }

    #[test]
    fn lists_every_allowed_value() {
        let values = ypso_basal().values();
        assert_eq!(values.first(), Some(&rate(20)));
        assert_eq!(values.last(), Some(&rate(2_500)));
        assert!(values.iter().all(|v| ypso_basal().allows(*v)));
        assert_eq!(values.len(), 98 + 50 + 6);
        let offset = Bounds::uniform(rate(15), rate(40), rate(10)).values();
        assert_eq!(offset, [rate(20), rate(30), rate(40)]);
    }

    #[test]
    fn a_zero_step_allows_nothing() {
        let bounds = Bounds::uniform(Percent::new(0), Percent::new(10), Percent::new(0));
        assert!(!bounds.allows(Percent::new(0)));
        assert_eq!(bounds.floor(Percent::new(5)), None);
        assert!(bounds.values().is_empty());
        let rates = Bounds::uniform(rate(0), rate(10), rate(0));
        assert_eq!(rates.floor(rate(5)), None);
        assert!(rates.values().is_empty());
    }

    #[test]
    fn checks_temp_basal_by_kind() {
        let limits = limits();
        assert!(limits.allows_temp_basal(TempBasalRate::Percent(Percent::new(150)), HALF_HOUR));
        assert!(!limits.allows_temp_basal(TempBasalRate::Percent(Percent::new(155)), HALF_HOUR));
        assert!(!limits.allows_temp_basal(
            TempBasalRate::Percent(Percent::new(150)),
            Duration::from_secs(60)
        ));
        let absolute = TempBasalRate::Absolute(rate(1_000));
        assert!(!limits.allows_temp_basal(absolute, HALF_HOUR));
        let with_absolute = Limits {
            temp_basal_absolute: Some(ypso_basal()),
            ..limits
        };
        assert!(with_absolute.allows_temp_basal(absolute, HALF_HOUR));
    }

    #[test]
    fn checks_extended_bolus() {
        let amount = mu(2_000);
        assert!(!limits().allows_extended_bolus(amount, HALF_HOUR));
        let extended = Limits {
            extended_bolus_duration: Some(limits().temp_basal_duration),
            ..limits()
        };
        assert!(extended.allows_extended_bolus(amount, HALF_HOUR));
        assert!(!extended.allows_extended_bolus(mu(2_050), HALF_HOUR));
    }
}
