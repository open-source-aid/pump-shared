use alloc::vec::Vec;

use crate::OpId;
use crate::{BleOp, BleResult, Class, KeyRef, Secret, Slot};

#[derive(Debug)]
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
    /// Ask the host for the public half of a key it holds.
    ///
    /// Some device protocols need the engine to send that public half to the
    /// device. It answers with [`Completion::PublicKey`].
    PublicKey {
        id: OpId,
        key: KeyRef,
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
            | Self::PublicKey { id, .. }
            | Self::Random { id, .. } => *id,
        }
    }
}

#[derive(Debug)]
pub enum Completion {
    Ble(BleResult),
    Stored,
    Loaded(Option<Vec<u8>>),
    Wrapped(Vec<u8>),
    Secret(Secret),
    /// The public half of a host-held key, 32 bytes.
    ///
    /// Plain bytes, not a [`Secret`]: a public key is meant to go on the wire
    /// to the device, so it needs neither zeroising nor a redacted `Debug`.
    /// `Secret` is reserved for material that must never leak, which is what
    /// makes its use in `Debug` output and logs meaningful (S10).
    PublicKey([u8; 32]),
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

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
            Action::PublicKey {
                id,
                key: KeyRef::new("k"),
            },
            Action::Random { id, len: 1 },
        ];
        for action in actions {
            assert_eq!(action.id(), id);
        }
    }

    #[test]
    fn public_key_completion_carries_plain_bytes() {
        let completion = Completion::PublicKey([7; 32]);
        assert!(matches!(completion, Completion::PublicKey(public) if public == [7; 32]));
        // Unlike `Secret`, it is printed in full: a public key is not secret.
        assert!(format!("{:?}", Completion::PublicKey([7; 32])).contains("7, 7"));
    }
}
