use core::time::Duration;

use crate::KeyRef;
use crate::{Address, InsulinAmount, Percent, Sequence};

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
        rate: Percent,
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
        }
    }

    pub fn is_stop_path(&self) -> bool {
        match self {
            Self::CancelTempBasal | Self::CancelExtendedBolus => true,
            Self::TempBasal { rate, .. } => rate.is_neutral(),
            _ => false,
        }
    }

    pub fn delivers_insulin(&self) -> bool {
        match self {
            Self::Bolus { .. } | Self::ExtendedBolus { .. } => true,
            Self::TempBasal { rate, .. } => !rate.is_neutral(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_temp_basal_is_a_stop_path() {
        let cancel = Command::TempBasal {
            rate: Percent::NEUTRAL,
            duration: Duration::from_secs(1800),
        };
        assert!(cancel.is_stop_path());
        assert!(!cancel.delivers_insulin());
    }

    #[test]
    fn bolus_delivers_insulin() {
        let bolus = Command::Bolus {
            amount: InsulinAmount::from_hundredths(100),
        };
        assert!(bolus.delivers_insulin());
        assert!(!bolus.is_stop_path());
        assert_eq!(bolus.kind(), CommandKind::Bolus);
    }

    #[test]
    fn every_command_has_its_kind() {
        let hour = Duration::from_secs(3600);
        let amount = InsulinAmount::from_hundredths(100);
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
                    rate: Percent::new(120),
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
            amount: InsulinAmount::from_hundredths(100),
            duration: Duration::from_secs(3600),
        };
        assert!(Command::CancelTempBasal.is_stop_path());
        assert!(Command::CancelExtendedBolus.is_stop_path());
        assert!(!Command::Status.is_stop_path());
        assert!(extended.delivers_insulin());
        assert!(!Command::Status.delivers_insulin());
    }
}
