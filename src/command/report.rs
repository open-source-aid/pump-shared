use alloc::vec::Vec;

use crate::{BasalProfile, HistoryEntry, InsulinAmount, PumpStatus, PumpTime};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Report {
    Done,
    Status(PumpStatus),
    Clock(PumpTime),
    Delivered(InsulinAmount),
    Profile(BasalProfile),
    History(Vec<HistoryEntry>),
}
