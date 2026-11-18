use core::time::Duration;

use alloc::vec::Vec;

use crate::{Alarm, BasalRate, InsulinAmount, Percent, TherapyState, UtcTime};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PumpTime(u64);

impl PumpTime {
    pub const fn from_local_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub const fn local_seconds(self) -> u64 {
        self.0
    }

    pub fn to_utc(self, utc_offset: i32) -> UtcTime {
        let utc = self.0.saturating_add_signed(-i64::from(utc_offset));
        UtcTime::from_unix_millis(utc.saturating_mul(1000))
    }

    pub fn drift(self, utc_offset: i32, now: UtcTime) -> Duration {
        let pump = self.to_utc(utc_offset).as_unix_millis();
        Duration::from_millis(pump.abs_diff(now.as_unix_millis()))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct PumpStatus {
    pub therapy: TherapyState,
    pub battery: Option<Percent>,
    pub reservoir: Option<InsulinAmount>,
    pub basal_rate: Option<BasalRate>,
    pub clock: Option<PumpTime>,
    pub delivery_uncertain: bool,
    pub alarms: Vec<Alarm>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_local_pump_time_to_utc() {
        let local = PumpTime::from_local_seconds(10_000);
        assert_eq!(local.local_seconds(), 10_000);
        assert_eq!(local.to_utc(3600).as_unix_millis(), 6_400_000);
        assert_eq!(local.to_utc(-3600).as_unix_millis(), 13_600_000);
        assert_eq!(
            PumpTime::from_local_seconds(10)
                .to_utc(3600)
                .as_unix_millis(),
            0
        );
    }

    #[test]
    fn measures_drift_either_way() {
        let pump = PumpTime::from_local_seconds(100);
        assert_eq!(
            pump.drift(0, UtcTime::from_unix_millis(90_000)),
            Duration::from_secs(10)
        );
        assert_eq!(
            pump.drift(0, UtcTime::from_unix_millis(110_000)),
            Duration::from_secs(10)
        );
    }

    #[test]
    fn unknown_status_has_no_readings() {
        let status = PumpStatus::default();
        assert_eq!(status.battery, None);
        assert_eq!(status.therapy, TherapyState::default());
        assert!(!status.delivery_uncertain);
        assert!(status.alarms.is_empty());
    }
}
