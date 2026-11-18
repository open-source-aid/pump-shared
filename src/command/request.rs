use core::time::Duration;

use crate::KeyRef;
use crate::{Address, BasalProfile, BasalRate, InsulinAmount, Sequence, TempBasalRate};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Command {
    Status,
    Pair {
        address: Address,
        server_key: Option<KeyRef>,
    },
    Unpair,
    Bolus {
        amount: InsulinAmount,
    },
    TempBasal {
        rate: TempBasalRate,
        duration: Duration,
    },
    CancelTempBasal,
    ExtendedBolus {
        amount: InsulinAmount,
        duration: Duration,
    },
    CancelExtendedBolus,
    ReadClock,
    SyncClock,
    ReadHistory {
        after: Sequence,
    },
    ReadBasalProfile,
    SetBasalProfile {
        profile: BasalProfile,
    },
    Suspend,
    Resume,
    SetDeliveryLimits {
        max_bolus: InsulinAmount,
        max_basal: BasalRate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CommandKind {
    Status,
    Pair,
    Unpair,
    Bolus,
    TempBasal,
    CancelTempBasal,
    ExtendedBolus,
    CancelExtendedBolus,
    ReadClock,
    SyncClock,
    ReadHistory,
    ReadBasalProfile,
    SetBasalProfile,
    Suspend,
    Resume,
    SetDeliveryLimits,
    Stop,
}

impl Command {
    pub fn kind(&self) -> CommandKind {
        match self {
            Self::Status => CommandKind::Status,
            Self::Pair { .. } => CommandKind::Pair,
            Self::Unpair => CommandKind::Unpair,
            Self::Bolus { .. } => CommandKind::Bolus,
            Self::TempBasal { .. } => CommandKind::TempBasal,
            Self::CancelTempBasal => CommandKind::CancelTempBasal,
            Self::ExtendedBolus { .. } => CommandKind::ExtendedBolus,
            Self::CancelExtendedBolus => CommandKind::CancelExtendedBolus,
            Self::ReadClock => CommandKind::ReadClock,
            Self::SyncClock => CommandKind::SyncClock,
            Self::ReadHistory { .. } => CommandKind::ReadHistory,
            Self::ReadBasalProfile => CommandKind::ReadBasalProfile,
            Self::SetBasalProfile { .. } => CommandKind::SetBasalProfile,
            Self::Suspend => CommandKind::Suspend,
            Self::Resume => CommandKind::Resume,
            Self::SetDeliveryLimits { .. } => CommandKind::SetDeliveryLimits,
        }
    }

    pub fn is_stop_path(&self) -> bool {
        match self {
            Self::CancelTempBasal | Self::CancelExtendedBolus | Self::Suspend => true,
            Self::TempBasal { rate, .. } => rate.is_neutral(),
            _ => false,
        }
    }

    pub fn delivers_insulin(&self) -> bool {
        match self {
            Self::Bolus { .. }
            | Self::ExtendedBolus { .. }
            | Self::SetBasalProfile { .. }
            | Self::Resume => true,
            Self::TempBasal { rate, .. } => !rate.is_neutral(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Percent;

    #[test]
    fn neutral_temp_basal_is_a_stop_path() {
        let cancel = Command::TempBasal {
            rate: TempBasalRate::Percent(Percent::NEUTRAL),
            duration: Duration::from_secs(1800),
        };
        assert!(cancel.is_stop_path());
        assert!(!cancel.delivers_insulin());
    }

    #[test]
    fn bolus_delivers_insulin() {
        let bolus = Command::Bolus {
            amount: InsulinAmount::from_milliunits(100),
        };
        assert!(bolus.delivers_insulin());
        assert!(!bolus.is_stop_path());
        assert_eq!(bolus.kind(), CommandKind::Bolus);
    }

    #[test]
    fn every_command_has_its_kind() {
        let hour = Duration::from_secs(3600);
        let amount = InsulinAmount::from_milliunits(100);
        let cases = [
            (Command::Status, CommandKind::Status),
            (
                Command::Pair {
                    address: Address::new([0; 6]),
                    server_key: None,
                },
                CommandKind::Pair,
            ),
            (Command::Unpair, CommandKind::Unpair),
            (Command::Bolus { amount }, CommandKind::Bolus),
            (
                Command::TempBasal {
                    rate: TempBasalRate::Percent(Percent::new(120)),
                    duration: hour,
                },
                CommandKind::TempBasal,
            ),
            (Command::CancelTempBasal, CommandKind::CancelTempBasal),
            (
                Command::ExtendedBolus {
                    amount,
                    duration: hour,
                },
                CommandKind::ExtendedBolus,
            ),
            (
                Command::CancelExtendedBolus,
                CommandKind::CancelExtendedBolus,
            ),
            (Command::ReadClock, CommandKind::ReadClock),
            (Command::SyncClock, CommandKind::SyncClock),
            (
                Command::ReadHistory {
                    after: Sequence::new(1),
                },
                CommandKind::ReadHistory,
            ),
        ];
        for (command, kind) in cases {
            assert_eq!(command.kind(), kind);
        }
    }

    #[test]
    fn classifies_stop_paths_and_delivery() {
        let extended = Command::ExtendedBolus {
            amount: InsulinAmount::from_milliunits(100),
            duration: Duration::from_secs(3600),
        };
        assert!(Command::CancelTempBasal.is_stop_path());
        assert!(Command::CancelExtendedBolus.is_stop_path());
        assert!(!Command::Status.is_stop_path());
        assert!(extended.delivers_insulin());
        assert!(!Command::Status.delivers_insulin());
    }

    #[test]
    fn profile_and_suspend_commands() {
        let profile = crate::BasalProfile::new(alloc::vec![crate::BasalSegment {
            start: Duration::ZERO,
            rate: crate::BasalRate::from_milliunits_per_hour(800),
        }])
        .unwrap();
        let set = Command::SetBasalProfile { profile };
        assert_eq!(set.kind(), CommandKind::SetBasalProfile);
        assert!(set.delivers_insulin());
        assert_eq!(
            Command::ReadBasalProfile.kind(),
            CommandKind::ReadBasalProfile
        );
        assert!(!Command::ReadBasalProfile.delivers_insulin());
        assert_eq!(Command::Suspend.kind(), CommandKind::Suspend);
        assert!(Command::Suspend.is_stop_path());
        assert_eq!(Command::Resume.kind(), CommandKind::Resume);
        assert!(Command::Resume.delivers_insulin());
        let limits = Command::SetDeliveryLimits {
            max_bolus: InsulinAmount::from_milliunits(10_000),
            max_basal: crate::BasalRate::from_milliunits_per_hour(3_000),
        };
        assert_eq!(limits.kind(), CommandKind::SetDeliveryLimits);
        assert!(!limits.delivers_insulin());
    }
}
