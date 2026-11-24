use alloc::vec::Vec;

use crate::{BasalProfile, HistoryEntry, InsulinAmount, PumpStatus, PumpTime, Sequence};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Report {
    Done,
    Status(PumpStatus),
    Clock(PumpTime),
    Delivered(InsulinAmount),
    Profile(BasalProfile),
    History {
        /// The entries in the requested page, newest first.
        entries: Vec<HistoryEntry>,
        /// The newest entry sequence the pump currently holds, independent of
        /// the page. The host resumes from it and, by comparing it against the
        /// page, tells whether the page caught up to the newest records or was
        /// bounded by [`Command::ReadHistory::limit`](crate::Command) and more
        /// recent reads remain.
        newest: Sequence,
    },
}
