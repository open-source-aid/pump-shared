use alloc::boxed::Box;

use crate::Engine;
use crate::{Address, CommandKind, NotifyKind};
use crate::{Class, Slot, Uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Kind {
    Pump,
    Cgm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BleMatch {
    pub name_prefixes: &'static [&'static str],
    pub service_uuids: &'static [Uuid],
    pub manufacturer_id: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GattChar {
    pub name: &'static str,
    pub service: Uuid,
    pub characteristic: Uuid,
    pub notify: Option<NotifyKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variant {
    pub id: &'static str,
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Feature {
    pub name: &'static str,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DriverInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: Kind,
    pub version: &'static str,
    pub api_version: u16,
    pub protocol: &'static str,
    pub ble: BleMatch,
    pub gatt: &'static [GattChar],
    pub variants: &'static [Variant],
    pub commands: &'static [CommandKind],
    pub features: &'static [Feature],
    pub slots: &'static [(Slot, Class)],
}

impl DriverInfo {
    pub fn has_feature(&self, name: &str) -> bool {
        self.features.iter().any(|f| f.name == name && f.enabled)
    }

    pub fn accepts(&self, kind: CommandKind) -> bool {
        self.commands.contains(&kind)
    }

    pub fn listens(&self, service: Uuid, characteristic: Uuid) -> Option<NotifyKind> {
        self.gatt
            .iter()
            .find(|c| c.service == service && c.characteristic == characteristic)
            .and_then(|c| c.notify)
    }

    pub fn listened(&self) -> impl Iterator<Item = &GattChar> {
        self.gatt.iter().filter(|c| c.notify.is_some())
    }

    pub fn is_compatible(&self) -> bool {
        self.api_version == crate::API_VERSION
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Advertisement<'a> {
    pub address: Address,
    pub name: Option<&'a str>,
    pub service_uuids: &'a [Uuid],
    pub manufacturer: Option<(u16, &'a [u8])>,
    pub rssi: i8,
}

impl BleMatch {
    pub fn matches(&self, adv: &Advertisement<'_>) -> bool {
        let name = self.name_prefixes.is_empty()
            || adv
                .name
                .is_some_and(|n| self.name_prefixes.iter().any(|p| n.starts_with(p)));
        let services = self.service_uuids.is_empty()
            || self
                .service_uuids
                .iter()
                .any(|u| adv.service_uuids.contains(u));
        let manufacturer = self
            .manufacturer_id
            .is_none_or(|id| adv.manufacturer.is_some_and(|(m, _)| m == id));
        name && services && manufacturer
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Config {
    pub address: Option<Address>,
    pub utc_offset_secs: i32,
}

pub trait Driver: Sync {
    fn info(&self) -> &'static DriverInfo;

    fn matches(&self, adv: &Advertisement<'_>) -> Option<&'static Variant>;

    fn new_engine(&self, variant: &'static Variant, config: Config) -> Box<dyn Engine>;
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERVICE: Uuid = Uuid::from_bytes([1; 16]);
    const PROGRESS: Uuid = Uuid::from_bytes([2; 16]);

    const MATCH: BleMatch = BleMatch {
        name_prefixes: &["Pump_"],
        service_uuids: &[SERVICE],
        manufacturer_id: None,
    };

    fn adv<'a>(name: Option<&'a str>, services: &'a [Uuid]) -> Advertisement<'a> {
        Advertisement {
            address: Address::new([0; 6]),
            name,
            service_uuids: services,
            manufacturer: None,
            rssi: -60,
        }
    }

    #[test]
    fn matches_name_and_service() {
        assert!(MATCH.matches(&adv(Some("Pump_123"), &[SERVICE])));
    }

    #[test]
    fn rejects_wrong_name() {
        assert!(!MATCH.matches(&adv(Some("Other"), &[SERVICE])));
    }

    #[test]
    fn rejects_missing_service() {
        assert!(!MATCH.matches(&adv(Some("Pump_123"), &[])));
    }

    const INFO: DriverInfo = DriverInfo {
        id: "test",
        name: "Test",
        kind: Kind::Pump,
        version: "0.0.0",
        api_version: crate::API_VERSION,
        protocol: "test-v1",
        ble: MATCH,
        gatt: &[
            GattChar {
                name: "status",
                service: SERVICE,
                characteristic: SERVICE,
                notify: None,
            },
            GattChar {
                name: "bolus progress",
                service: SERVICE,
                characteristic: PROGRESS,
                notify: Some(NotifyKind::Indication),
            },
        ],
        variants: &[],
        commands: &[CommandKind::Status],
        features: &[
            Feature {
                name: "delivery",
                enabled: false,
            },
            Feature {
                name: "pairing",
                enabled: true,
            },
        ],
        slots: &[(Slot::Counters, Class::Monotonic)],
    };

    #[test]
    fn describes_compiled_features_and_commands() {
        assert!(INFO.has_feature("pairing"));
        assert!(!INFO.has_feature("delivery"));
        assert!(!INFO.has_feature("missing"));
        assert!(INFO.accepts(CommandKind::Status));
        assert!(!INFO.accepts(CommandKind::Bolus));
        assert!(INFO.is_compatible());
    }

    #[test]
    fn matches_manufacturer() {
        let by_maker = BleMatch {
            name_prefixes: &[],
            service_uuids: &[],
            manufacturer_id: Some(0x0123),
        };
        let mut found = adv(None, &[]);
        assert!(!by_maker.matches(&found));
        found.manufacturer = Some((0x0123, &[]));
        assert!(by_maker.matches(&found));
    }

    #[test]
    fn announces_what_it_listens_to() {
        assert_eq!(
            INFO.listens(SERVICE, PROGRESS),
            Some(NotifyKind::Indication)
        );
        assert_eq!(INFO.listens(SERVICE, SERVICE), None);
        assert_eq!(INFO.listens(PROGRESS, PROGRESS), None);
        let listened: alloc::vec::Vec<_> = INFO.listened().map(|c| c.name).collect();
        assert_eq!(listened, ["bolus progress"]);
    }
}
