use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use crate::{BasalRate, InsulinAmount, PumpTime, Sequence, TempBasalRate, UtcTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BolusKind {
    Normal,
    Smb,
    Priming,
    Extended,
    Combined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TempBasalPhase {
    Started,
    Running,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Component {
    Reservoir,
    Cannula,
    Tubing,
    Battery,
    Pump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AlarmKind {
    Occlusion,
    ReservoirLow,
    ReservoirEmpty,
    BatteryLow,
    BatteryEmpty,
    AutoOff,
    NoDelivery,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LimitChange {
    MaxBolus {
        value: InsulinAmount,
        previous: InsulinAmount,
    },
    MaxBasal {
        value: BasalRate,
        previous: BasalRate,
    },
    BolusStep {
        value: InsulinAmount,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Alarm {
    pub kind: AlarmKind,
    pub code: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HistoryEvent {
    Bolus {
        kind: BolusKind,
        requested: InsulinAmount,
        delivered: InsulinAmount,
        duration: Duration,
        automatic: bool,
        in_progress: bool,
    },
    ExtendedBolusEnded {
        delivered: InsulinAmount,
    },
    TempBasal {
        rate: TempBasalRate,
        duration: Duration,
        phase: TempBasalPhase,
    },
    Suspended,
    Resumed,
    BasalProfileChanged,
    Rewound,
    Replaced(Component),
    ClockChanged {
        previous: PumpTime,
    },
    Alarm(Alarm),
    AlarmCleared(Alarm),
    DailyTotal {
        day: PumpTime,
        amount: InsulinAmount,
    },
    LimitChanged(LimitChange),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HistoryEntry {
    pub sequence: Sequence,
    pub time: UtcTime,
    pub pump_time: Option<PumpTime>,
    pub event: HistoryEvent,
    pub raw: Vec<u8>,
}

impl HistoryEntry {
    pub fn sync_id(&self, serial: &str) -> String {
        format!("{serial}-{}", self.sequence.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_is_distinct_from_extended() {
        assert_ne!(BolusKind::Combined, BolusKind::Extended);
    }

    #[test]
    fn temp_basal_carries_its_phase() {
        for phase in [
            TempBasalPhase::Started,
            TempBasalPhase::Running,
            TempBasalPhase::Ended,
        ] {
            let event = HistoryEvent::TempBasal {
                rate: TempBasalRate::Percent(crate::Percent::NEUTRAL),
                duration: Duration::from_secs(1_800),
                phase,
            };
            assert!(matches!(
                event,
                HistoryEvent::TempBasal { phase: p, .. } if p == phase
            ));
        }
        assert_ne!(TempBasalPhase::Started, TempBasalPhase::Ended);
    }

    #[test]
    fn sync_id_combines_serial_and_sequence() {
        let entry = HistoryEntry {
            sequence: Sequence::new(42),
            time: UtcTime::from_unix_millis(0),
            pump_time: None,
            event: HistoryEvent::Replaced(Component::Cannula),
            raw: Vec::new(),
        };
        assert_eq!(entry.sync_id("10155555"), "10155555-42");
    }
}
