use alloc::vec::Vec;

use crate::OpId;
use crate::{BleOp, BleResult, Class, KeyRef, Secret, Slot};

#[derive(Debug)]
#[non_exhaustive]
pub enum Action {
    Ble {
        id: OpId,
        op: BleOp,
    },
    Store {
        id: OpId,
        slot: Slot,
        class: Class,
        bytes: Vec<u8>,
    },
    Load {
        id: OpId,
        slot: Slot,
    },
    Erase {
        id: OpId,
        slot: Slot,
    },
    Wrap {
        id: OpId,
        plaintext: Secret,
    },
    Unwrap {
        id: OpId,
        wrapped: Vec<u8>,
    },
    KeyAgree {
        id: OpId,
        key: KeyRef,
        peer_public: [u8; 32],
    },
    Random {
        id: OpId,
        len: usize,
    },
}

impl Action {
    pub fn id(&self) -> OpId {
        match self {
            Self::Ble { id, .. }
            | Self::Store { id, .. }
            | Self::Load { id, .. }
            | Self::Erase { id, .. }
            | Self::Wrap { id, .. }
            | Self::Unwrap { id, .. }
            | Self::KeyAgree { id, .. }
            | Self::Random { id, .. } => *id,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Completion {
    Ble(BleResult),
    Stored,
    Loaded(Option<Vec<u8>>),
    Wrapped(Vec<u8>),
    Secret(Secret),
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_its_id() {
        let id = OpId::new(9);
        let actions = [
            Action::Ble {
                id,
                op: BleOp::Disconnect,
            },
            Action::Store {
                id,
                slot: Slot::Counters,
                class: Class::Monotonic,
                bytes: Vec::new(),
            },
            Action::Load {
                id,
                slot: Slot::Counters,
            },
            Action::Erase {
                id,
                slot: Slot::Counters,
            },
            Action::Wrap {
                id,
                plaintext: Secret::new(Vec::new()),
            },
            Action::Unwrap {
                id,
                wrapped: Vec::new(),
            },
            Action::KeyAgree {
                id,
                key: KeyRef::new("k"),
                peer_public: [0; 32],
            },
            Action::Random { id, len: 1 },
        ];
        for action in actions {
            assert_eq!(action.id(), id);
        }
    }
}
