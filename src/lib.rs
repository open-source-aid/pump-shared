#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod command;
pub mod device;
pub mod engine;
pub mod history;
pub mod therapy;

pub use command::{Approximation, Command, CommandKind, Failure, Outcome, Refusal, Report};
#[cfg(feature = "sim")]
pub use device::SimDevice;
pub use device::{
    grid, Address, Advertisement, BleMatch, BleOp, BleResult, Bounds, Capabilities, Config,
    DeviceInfo, Driver, DriverInfo, DriverMatch, EngineState, Feature, GattChar, Kind, Limits,
    Nominal, NominalBounds, NominalCapabilities, NominalLimits, Notification, NotifyKind,
    PumpStatus, PumpTime, Sequence, Tier, Uuid, Variant,
};
pub use engine::{
    Action, Class, CommandId, Completion, CompletionError, Engine, Event, KeyRef, OpId, Secret,
    Slot, SubmitError, Timestamp, UtcTime,
};
pub use history::{
    Alarm, AlarmKind, BolusKind, Component, HistoryEntry, HistoryEvent, LimitChange, TempBasalPhase,
};
pub use therapy::{
    Basal, BasalProfile, BasalRate, BasalSegment, Bolus, Extended, InsulinAmount, Percent,
    ProfileError, PumpMode, Quantity, TempBasalRate, TherapyMachine, TherapyState, Transition,
    TransitionError, DAY,
};
#[cfg(feature = "graph")]
pub use therapy::{Edge, Graph, Requirement};

pub const API_VERSION: u16 = 3;
