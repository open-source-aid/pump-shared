use crate::{BleOp, BleResult, KeyRef, Notification, Timestamp};

pub trait SimDevice {
    fn handle(&mut self, op: &BleOp, now: Timestamp) -> BleResult;

    fn key_agree(&mut self, _key: &KeyRef, _peer_public: &[u8; 32]) -> Option<[u8; 32]> {
        None
    }

    fn poll_notification(&mut self, _now: Timestamp) -> Option<Notification> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Silent;

    impl SimDevice for Silent {
        fn handle(&mut self, _op: &BleOp, _now: Timestamp) -> BleResult {
            BleResult::Done
        }
    }

    #[test]
    fn defaults_neither_agree_keys_nor_notify() {
        let mut device = Silent;
        let now = Timestamp::default();
        assert_eq!(device.handle(&BleOp::Disconnect, now), BleResult::Done);
        assert_eq!(device.key_agree(&KeyRef::new("k"), &[0; 32]), None);
        assert_eq!(device.poll_notification(now), None);
    }
}
