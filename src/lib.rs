#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod command;
pub mod device;
pub mod engine;
#[cfg(feature = "testkit")]
pub mod testkit;
pub mod therapy;

pub use command::{Command, CommandKind, Failure, Outcome, Refusal};
pub use device::{
    Address, Advertisement, BleMatch, BleOp, BleResult, Config, DeviceInfo, Driver, DriverInfo,
    EngineState, Feature, GattChar, Kind, Sequence, Uuid, Variant,
};
pub use engine::{
    Action, Class, CommandId, Completion, CompletionError, Engine, Event, KeyRef, OpId, Secret,
    Slot, SubmitError, Timestamp, UtcTime,
};
pub use therapy::{
    Basal, Bolus, Extended, Graph, InsulinAmount, Percent, PumpMode, TherapyMachine, TherapyState,
    Transition, TransitionError,
};

pub const API_VERSION: u16 = 1;
