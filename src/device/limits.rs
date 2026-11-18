use core::time::Duration;

use crate::{BasalRate, InsulinAmount, Percent, Quantity, TempBasalRate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bounds<T> {
    pub min: T,
    pub max: T,
    pub step: T,
}

impl<T: Quantity> Bounds<T> {
    pub fn allows(&self, value: T) -> bool {
        let step = self.step.raw();
        value >= self.min && value <= self.max && step != 0 && value.raw() % step == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            TempBasalRate::Percent(p) => self.temp_basal_percent.is_some_and(|b| b.allows(p)),
            TempBasalRate::Absolute(r) => self.temp_basal_absolute.is_some_and(|b| b.allows(r)),
        };
        rate_ok && self.temp_basal_duration.allows(duration)
    }

    pub fn allows_extended_bolus(&self, amount: InsulinAmount, duration: Duration) -> bool {
        self.bolus.allows(amount)
            && self
                .extended_bolus_duration
                .is_some_and(|b| b.allows(duration))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HALF_HOUR: Duration = Duration::from_secs(1800);

    fn limits() -> Limits {
        Limits {
            bolus: Bounds {
                min: InsulinAmount::from_milliunits(100),
                max: InsulinAmount::from_milliunits(25_000),
                step: InsulinAmount::from_milliunits(100),
            },
            basal: Bounds {
                min: BasalRate::ZERO,
                max: BasalRate::from_milliunits_per_hour(40_000),
                step: BasalRate::from_milliunits_per_hour(10),
            },
            temp_basal_percent: Some(Bounds {
                min: Percent::new(0),
                max: Percent::new(200),
                step: Percent::new(10),
            }),
            temp_basal_absolute: None,
            temp_basal_duration: Bounds {
                min: HALF_HOUR,
                max: Duration::from_secs(24 * 3600),
                step: HALF_HOUR,
            },
            extended_bolus_duration: None,
        }
    }

    #[test]
    fn checks_range_and_step() {
        let bolus = limits().bolus;
        assert!(bolus.allows(InsulinAmount::from_milliunits(1_500)));
        assert!(!bolus.allows(InsulinAmount::from_milliunits(1_525)));
        assert!(!bolus.allows(InsulinAmount::from_milliunits(30_000)));
        assert!(!bolus.allows(InsulinAmount::ZERO));
    }

    #[test]
    fn a_zero_step_allows_nothing() {
        let bounds = Bounds {
            min: Percent::new(0),
            max: Percent::new(10),
            step: Percent::new(0),
        };
        assert!(!bounds.allows(Percent::new(0)));
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
        let absolute = TempBasalRate::Absolute(BasalRate::from_milliunits_per_hour(1_000));
        assert!(!limits.allows_temp_basal(absolute, HALF_HOUR));
        let with_absolute = Limits {
            temp_basal_absolute: Some(limits.basal),
            ..limits
        };
        assert!(with_absolute.allows_temp_basal(absolute, HALF_HOUR));
    }

    #[test]
    fn checks_extended_bolus() {
        let amount = InsulinAmount::from_milliunits(2_000);
        assert!(!limits().allows_extended_bolus(amount, HALF_HOUR));
        let extended = Limits {
            extended_bolus_duration: Some(limits().temp_basal_duration),
            ..limits()
        };
        assert!(extended.allows_extended_bolus(amount, HALF_HOUR));
        assert!(!extended.allows_extended_bolus(InsulinAmount::from_milliunits(2_050), HALF_HOUR));
    }
}
