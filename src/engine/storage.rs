#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Slot {
    Session,
    PendingSession,
    Counters,
    NonceState,
    Driver(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Class {
    Secret,
    Monotonic,
    Plain,
}
