use alloc::vec::Vec;
use core::time::Duration;

use crate::{InsulinAmount, PumpTime, Sequence, TempBasalRate, UtcTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BolusKind {
    Immediate,
    Extended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HistoryEvent {
    Bolus {
        kind: BolusKind,
        requested: InsulinAmount,
        delivered: InsulinAmount,
        duration: Duration,
    },
    TempBasal {
        rate: TempBasalRate,
        duration: Duration,
    },
    TempBasalEnded,
    Suspended,
    Resumed,
    BasalProfileChanged,
    Primed {
        amount: InsulinAmount,
    },
    CartridgeChanged,
    BatteryChanged,
    ClockChanged {
        previous: PumpTime,
    },
    Alarm {
        code: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HistoryEntry {
    pub sequence: Sequence,
    pub time: UtcTime,
    pub pump_time: Option<PumpTime>,
    pub event: HistoryEvent,
    pub raw: Vec<u8>,
}
