use alloc::boxed::Box;
use alloc::string::String;
use core::fmt;

use crate::{Action, Completion, Notification};

use crate::{
    Command, DeviceInfo, EngineState, InsulinAmount, Outcome, TherapyMachine, TherapyState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpId(u32);

impl OpId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandId(u32);

impl CommandId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcTime(u64);

impl UtcTime {
    pub const fn from_unix_millis(millis: u64) -> Self {
        Self(millis)
    }

    pub const fn as_unix_millis(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Event {
    Started {
        id: CommandId,
    },
    Progress {
        id: CommandId,
        delivered: InsulinAmount,
        requested: InsulinAmount,
    },
    Finished {
        id: CommandId,
        outcome: Outcome,
    },
    DeviceInfo(Box<DeviceInfo>),
    StateChanged(EngineState),
    TherapyChanged(TherapyState),
    Log {
        level: u8,
        line: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SubmitError {
    QueueFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CompletionError {
    NotPending,
    Mismatch,
}

impl fmt::Display for SubmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("command queue is full")
    }
}

impl fmt::Display for CompletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotPending => "no pending operation with this id",
            Self::Mismatch => "completion does not match the pending operation",
        })
    }
}

impl core::error::Error for SubmitError {}
impl core::error::Error for CompletionError {}

pub trait Engine: TherapyMachine {
    fn submit(&mut self, command: Command) -> Result<CommandId, SubmitError>;

    fn stop(&mut self) -> CommandId;

    fn handle_completion(
        &mut self,
        id: OpId,
        completion: Completion,
    ) -> Result<(), CompletionError>;

    fn handle_timeout(&mut self, now: Timestamp);

    fn handle_wall_clock(&mut self, now: UtcTime);

    fn handle_link_lost(&mut self);

    fn handle_notification(&mut self, notification: Notification);

    fn poll_action(&mut self) -> Option<Action>;

    fn poll_event(&mut self) -> Option<Event>;

    fn poll_timeout(&self) -> Option<Timestamp>;

    fn state(&self) -> EngineState;
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn exposes_raw_values() {
        assert_eq!(OpId::new(3).get(), 3);
        assert_eq!(CommandId::new(4).get(), 4);
        assert_eq!(Timestamp::from_millis(5).as_millis(), 5);
        assert_eq!(UtcTime::from_unix_millis(6).as_unix_millis(), 6);
    }

    #[test]
    fn errors_have_messages() {
        assert!(!SubmitError::QueueFull.to_string().is_empty());
        assert!(!CompletionError::NotPending.to_string().is_empty());
        assert!(!CompletionError::Mismatch.to_string().is_empty());
    }
}
