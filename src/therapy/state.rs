use alloc::boxed::Box;
use core::fmt;
use core::time::Duration;

use crate::{Command, InsulinAmount, Refusal, TempBasalRate};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PumpMode {
    #[default]
    Unknown,
    Off,
    Suspended,
    Running,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Basal {
    #[default]
    Unknown,
    Scheduled,
    Temporary {
        rate: TempBasalRate,
        duration: Duration,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Bolus {
    #[default]
    Unknown,
    Idle,
    Immediate {
        requested: InsulinAmount,
        delivered: InsulinAmount,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Extended {
    #[default]
    Unknown,
    Idle,
    Running {
        amount: InsulinAmount,
        duration: Duration,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TherapyState {
    pub mode: PumpMode,
    pub basal: Basal,
    pub bolus: Bolus,
    pub extended: Extended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Transition {
    Observed(TherapyState),
    Suspended,
    Resumed,
    TurnedOff,
    BolusStarted {
        requested: InsulinAmount,
    },
    BolusProgressed {
        delivered: InsulinAmount,
    },
    BolusEnded,
    TempBasalStarted {
        rate: TempBasalRate,
        duration: Duration,
    },
    TempBasalEnded,
    ExtendedStarted {
        amount: InsulinAmount,
        duration: Duration,
    },
    ExtendedEnded,
    LinkLost,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionError(Box<(TherapyState, Transition)>);

impl TransitionError {
    pub fn from_state(&self) -> &TherapyState {
        &self.0 .0
    }

    pub fn transition(&self) -> &Transition {
        &self.0 .1
    }
}

impl PumpMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Off => "Off",
            Self::Suspended => "Suspended",
            Self::Running => "Running",
        }
    }
}

impl Basal {
    pub fn name(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Scheduled => "Scheduled",
            Self::Temporary { .. } => "Temporary",
        }
    }
}

impl Bolus {
    pub fn name(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Idle => "Idle",
            Self::Immediate { .. } => "Immediate",
        }
    }
}

impl Extended {
    pub fn name(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Idle => "Idle",
            Self::Running { .. } => "Running",
        }
    }
}

impl Transition {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Observed(_) => "Observed",
            Self::Suspended => "Suspended",
            Self::Resumed => "Resumed",
            Self::TurnedOff => "TurnedOff",
            Self::BolusStarted { .. } => "BolusStarted",
            Self::BolusProgressed { .. } => "BolusProgressed",
            Self::BolusEnded => "BolusEnded",
            Self::TempBasalStarted { .. } => "TempBasalStarted",
            Self::TempBasalEnded => "TempBasalEnded",
            Self::ExtendedStarted { .. } => "ExtendedStarted",
            Self::ExtendedEnded => "ExtendedEnded",
            Self::LinkLost => "LinkLost",
        }
    }
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "transition {:?} not allowed from {:?}",
            self.transition(),
            self.from_state()
        )
    }
}

impl core::error::Error for TransitionError {}

impl TherapyState {
    pub fn guard(&self, command: &Command) -> Result<(), Refusal> {
        if command.is_stop_path() || !command.delivers_insulin() {
            return Ok(());
        }
        if *command == Command::Resume {
            return match self.mode {
                PumpMode::Suspended => Ok(()),
                _ => Err(Refusal::NotSuspended),
            };
        }
        if self.mode != PumpMode::Running {
            return Err(Refusal::NotRunning);
        }
        match command {
            Command::Bolus { .. } if self.bolus != Bolus::Idle => Err(Refusal::Busy),
            Command::ExtendedBolus { .. } if self.extended != Extended::Idle => Err(Refusal::Busy),
            _ => Ok(()),
        }
    }

    pub fn apply(&mut self, transition: Transition) -> Result<(), TransitionError> {
        let next = self
            .next(transition)
            .ok_or_else(|| TransitionError(Box::new((*self, transition))))?;
        *self = next;
        Ok(())
    }

    fn next(&self, transition: Transition) -> Option<Self> {
        let mut next = *self;
        match transition {
            Transition::Observed(observed) => next = observed,
            Transition::Suspended => {
                next.mode = PumpMode::Suspended;
                next.bolus = Bolus::Idle;
                next.extended = Extended::Idle;
                next.basal = Basal::Scheduled;
            }
            Transition::Resumed => {
                if self.mode != PumpMode::Suspended {
                    return None;
                }
                next.mode = PumpMode::Running;
            }
            Transition::TurnedOff => {
                next = Self {
                    mode: PumpMode::Off,
                    ..Self::default()
                }
            }
            Transition::BolusStarted { requested } => {
                if self.mode != PumpMode::Running || self.bolus != Bolus::Idle {
                    return None;
                }
                next.bolus = Bolus::Immediate {
                    requested,
                    delivered: InsulinAmount::ZERO,
                };
            }
            Transition::BolusProgressed { delivered } => match self.bolus {
                Bolus::Immediate {
                    requested,
                    delivered: before,
                } if delivered >= before => {
                    next.bolus = Bolus::Immediate {
                        requested,
                        delivered,
                    };
                }
                _ => return None,
            },
            Transition::BolusEnded => next.bolus = Bolus::Idle,
            Transition::TempBasalStarted { rate, duration } => {
                if self.mode != PumpMode::Running {
                    return None;
                }
                next.basal = if rate.is_neutral() {
                    Basal::Scheduled
                } else {
                    Basal::Temporary { rate, duration }
                };
            }
            Transition::TempBasalEnded => next.basal = Basal::Scheduled,
            Transition::ExtendedStarted { amount, duration } => {
                if self.mode != PumpMode::Running || self.extended != Extended::Idle {
                    return None;
                }
                next.extended = Extended::Running { amount, duration };
            }
            Transition::ExtendedEnded => next.extended = Extended::Idle,
            Transition::LinkLost => {
                if matches!(self.bolus, Bolus::Immediate { .. }) {
                    next.bolus = Bolus::Unknown;
                }
                if matches!(self.extended, Extended::Running { .. }) {
                    next.extended = Extended::Unknown;
                }
            }
        }
        Some(next)
    }
}

pub trait TherapyMachine {
    fn therapy(&self) -> &TherapyState;

    fn therapy_mut(&mut self) -> &mut TherapyState;

    fn guard(&self, command: &Command) -> Result<(), Refusal> {
        self.therapy().guard(command)
    }

    fn apply(&mut self, transition: Transition) -> Result<(), TransitionError> {
        self.therapy_mut().apply(transition)
    }
}

impl TherapyMachine for TherapyState {
    fn therapy(&self) -> &TherapyState {
        self
    }

    fn therapy_mut(&mut self) -> &mut TherapyState {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Percent;

    fn running() -> TherapyState {
        TherapyState {
            mode: PumpMode::Running,
            basal: Basal::Scheduled,
            bolus: Bolus::Idle,
            extended: Extended::Idle,
        }
    }

    fn units(hundredths: u32) -> InsulinAmount {
        InsulinAmount::from_milliunits(hundredths)
    }

    #[test]
    fn unknown_state_refuses_delivery_but_allows_stop() {
        let state = TherapyState::default();
        assert_eq!(
            state.guard(&Command::Bolus { amount: units(100) }),
            Err(Refusal::NotRunning)
        );
        assert_eq!(state.guard(&Command::CancelTempBasal), Ok(()));
        assert_eq!(state.guard(&Command::Status), Ok(()));
    }

    #[test]
    fn bolus_lifecycle() {
        let mut state = running();
        state
            .apply(Transition::BolusStarted {
                requested: units(150),
            })
            .unwrap();
        assert_eq!(
            state.guard(&Command::Bolus { amount: units(50) }),
            Err(Refusal::Busy)
        );
        state
            .apply(Transition::BolusProgressed {
                delivered: units(80),
            })
            .unwrap();
        assert_eq!(
            state.bolus,
            Bolus::Immediate {
                requested: units(150),
                delivered: units(80)
            }
        );
        state.apply(Transition::BolusEnded).unwrap();
        assert_eq!(state.bolus, Bolus::Idle);
    }

    #[test]
    fn progress_never_goes_backwards() {
        let mut state = running();
        state
            .apply(Transition::BolusStarted {
                requested: units(150),
            })
            .unwrap();
        state
            .apply(Transition::BolusProgressed {
                delivered: units(80),
            })
            .unwrap();
        let before = state;
        assert!(state
            .apply(Transition::BolusProgressed {
                delivered: units(40)
            })
            .is_err());
        assert_eq!(state, before);
    }

    #[test]
    fn suspended_pump_refuses_bolus_start() {
        let mut state = running();
        state.apply(Transition::Suspended).unwrap();
        assert!(state
            .apply(Transition::BolusStarted {
                requested: units(100)
            })
            .is_err());
        state.apply(Transition::Resumed).unwrap();
        assert!(state
            .apply(Transition::BolusStarted {
                requested: units(100)
            })
            .is_ok());
    }

    #[test]
    fn neutral_temp_basal_returns_to_schedule() {
        let mut state = running();
        let half_hour = Duration::from_secs(1800);
        state
            .apply(Transition::TempBasalStarted {
                rate: TempBasalRate::Percent(Percent::new(150)),
                duration: half_hour,
            })
            .unwrap();
        state
            .apply(Transition::TempBasalStarted {
                rate: TempBasalRate::Percent(Percent::NEUTRAL),
                duration: half_hour,
            })
            .unwrap();
        assert_eq!(state.basal, Basal::Scheduled);
    }

    #[test]
    fn link_lost_mid_bolus_makes_bolus_unknown() {
        let mut state = running();
        state
            .apply(Transition::BolusStarted {
                requested: units(100),
            })
            .unwrap();
        state.apply(Transition::LinkLost).unwrap();
        assert_eq!(state.bolus, Bolus::Unknown);
        assert_eq!(
            state.guard(&Command::Bolus { amount: units(100) }),
            Err(Refusal::Busy)
        );
    }

    struct NoExtended(TherapyState);

    impl TherapyMachine for NoExtended {
        fn therapy(&self) -> &TherapyState {
            &self.0
        }

        fn therapy_mut(&mut self) -> &mut TherapyState {
            &mut self.0
        }

        fn guard(&self, command: &Command) -> Result<(), Refusal> {
            if matches!(command, Command::ExtendedBolus { .. }) {
                return Err(Refusal::Unsupported);
            }
            self.0.guard(command)
        }
    }

    #[test]
    fn driver_can_override_guard() {
        let mut machine = NoExtended(running());
        machine.apply(Transition::TempBasalEnded).unwrap();
        assert_eq!(machine.therapy().basal, Basal::Scheduled);
        let extended = Command::ExtendedBolus {
            amount: units(100),
            duration: Duration::from_secs(3600),
        };
        assert_eq!(machine.guard(&extended), Err(Refusal::Unsupported));
        assert_eq!(
            machine.guard(&Command::Bolus { amount: units(100) }),
            Ok(())
        );
    }

    #[test]
    fn extended_bolus_lifecycle() {
        let mut state = running();
        let hour = Duration::from_secs(3600);
        let extended = Command::ExtendedBolus {
            amount: units(100),
            duration: hour,
        };
        state
            .apply(Transition::ExtendedStarted {
                amount: units(100),
                duration: hour,
            })
            .unwrap();
        assert_eq!(state.guard(&extended), Err(Refusal::Busy));
        assert!(state
            .apply(Transition::ExtendedStarted {
                amount: units(100),
                duration: hour
            })
            .is_err());
        state.apply(Transition::LinkLost).unwrap();
        assert_eq!(state.extended, Extended::Unknown);
        state.apply(Transition::ExtendedEnded).unwrap();
        assert_eq!(state.guard(&extended), Ok(()));
    }

    #[test]
    fn temp_basal_needs_a_running_pump() {
        let half_hour = Duration::from_secs(1800);
        let start = Transition::TempBasalStarted {
            rate: TempBasalRate::Percent(Percent::new(150)),
            duration: half_hour,
        };
        assert!(TherapyState::default().apply(start).is_err());
        let mut state = running();
        state.apply(start).unwrap();
        state.apply(Transition::TempBasalEnded).unwrap();
        assert_eq!(state.basal, Basal::Scheduled);
    }

    #[test]
    fn observation_replaces_everything() {
        let mut state = TherapyState::default();
        state.apply(Transition::Observed(running())).unwrap();
        assert_eq!(state, running());
    }

    #[test]
    fn turning_off_resets_everything() {
        let mut state = running();
        state.apply(Transition::TurnedOff).unwrap();
        assert_eq!(state.mode, PumpMode::Off);
        assert_eq!(state.bolus, Bolus::Unknown);
    }

    #[test]
    fn resume_needs_a_suspended_pump() {
        assert!(running().apply(Transition::Resumed).is_err());
    }

    #[test]
    fn progress_needs_a_running_bolus() {
        assert!(running()
            .apply(Transition::BolusProgressed {
                delivered: units(10)
            })
            .is_err());
    }

    #[test]
    fn rejected_transition_is_reported() {
        let error = running().apply(Transition::Resumed).unwrap_err();
        assert_eq!(*error.from_state(), running());
        assert_eq!(*error.transition(), Transition::Resumed);
        assert!(error.to_string().contains("Resumed"));
    }

    #[test]
    fn every_transition_has_a_name() {
        let hour = Duration::from_secs(3600);
        let transitions = [
            Transition::Observed(running()),
            Transition::Suspended,
            Transition::Resumed,
            Transition::TurnedOff,
            Transition::BolusStarted {
                requested: units(1),
            },
            Transition::BolusProgressed {
                delivered: units(1),
            },
            Transition::BolusEnded,
            Transition::TempBasalStarted {
                rate: TempBasalRate::Percent(Percent::NEUTRAL),
                duration: hour,
            },
            Transition::TempBasalEnded,
            Transition::ExtendedStarted {
                amount: units(1),
                duration: hour,
            },
            Transition::ExtendedEnded,
            Transition::LinkLost,
        ];
        let names: alloc::collections::BTreeSet<_> =
            transitions.iter().map(Transition::name).collect();
        assert_eq!(names.len(), transitions.len());
    }

    #[test]
    fn state_is_its_own_machine() {
        let mut state = running();
        assert_eq!(TherapyMachine::guard(&state, &Command::Status), Ok(()));
        TherapyMachine::apply(&mut state, Transition::Suspended).unwrap();
        assert_eq!(TherapyMachine::therapy(&state).mode, PumpMode::Suspended);
    }

    #[test]
    fn every_region_state_has_a_name() {
        let hour = Duration::from_secs(3600);
        let modes = [
            PumpMode::Unknown,
            PumpMode::Off,
            PumpMode::Suspended,
            PumpMode::Running,
        ];
        assert_eq!(
            modes.map(PumpMode::name),
            ["Unknown", "Off", "Suspended", "Running"]
        );
        let basals = [
            Basal::Unknown,
            Basal::Scheduled,
            Basal::Temporary {
                rate: TempBasalRate::Percent(Percent::new(150)),
                duration: hour,
            },
        ];
        assert_eq!(
            basals.map(Basal::name),
            ["Unknown", "Scheduled", "Temporary"]
        );
        let boluses = [
            Bolus::Unknown,
            Bolus::Idle,
            Bolus::Immediate {
                requested: units(1),
                delivered: units(0),
            },
        ];
        assert_eq!(boluses.map(Bolus::name), ["Unknown", "Idle", "Immediate"]);
        let extendeds = [
            Extended::Unknown,
            Extended::Idle,
            Extended::Running {
                amount: units(1),
                duration: hour,
            },
        ];
        assert_eq!(
            extendeds.map(Extended::name),
            ["Unknown", "Idle", "Running"]
        );
    }

    #[test]
    fn resume_needs_a_suspended_pump_and_suspend_is_never_refused() {
        let mut state = running();
        assert_eq!(state.guard(&Command::Resume), Err(Refusal::NotSuspended));
        assert_eq!(TherapyState::default().guard(&Command::Suspend), Ok(()));
        state.apply(Transition::Suspended).unwrap();
        assert_eq!(state.guard(&Command::Resume), Ok(()));
    }
}
