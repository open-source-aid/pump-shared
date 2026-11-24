use alloc::boxed::Box;

use crate::Engine;
use crate::Nominal;
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

/// One model inside the family a driver covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variant {
    pub id: &'static str,
    pub name: &'static str,
    /// What this model, specifically, is built for — when it differs from the
    /// family's [`DriverInfo::nominal`]. `None` means the family's figures
    /// hold for this model too.
    ///
    /// Read it through [`DriverInfo::nominal_for`], never directly: a `None`
    /// here means "the same as the family", not "nothing is known".
    pub nominal: Option<&'static Nominal>,
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
    /// What the device family is built for, before any device is connected:
    /// the grid to draw a setup screen from. **Not** what a connected device
    /// accepts — that is [`DeviceInfo::limits`](crate::DeviceInfo), read from
    /// the device itself, and the only thing a dose may be checked against.
    /// See [`Nominal`].
    pub nominal: Nominal,
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

    /// The nominal declaration that applies to `variant`: the model's own
    /// when it has one, the family's otherwise.
    pub fn nominal_for(&self, variant: &Variant) -> &Nominal {
        variant.nominal.unwrap_or(&self.nominal)
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

/// A driver's positive match on an advertisement: the variant it recognised, the driver's own id (so
/// a registry polling several drivers knows which one matched), and the canonical device address the
/// driver derives from the advertisement — `None` when it has nothing better than the
/// advertisement's own [`Advertisement::address`]. A serial encoded in the advertised name, for
/// example, yields the real MAC that a platform like iOS never exposes, so the harness can bind on it
/// without knowing anything about the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DriverMatch {
    pub driver_id: &'static str,
    pub variant: &'static Variant,
    pub address: Option<Address>,
}

pub trait Driver: Sync {
    fn info(&self) -> &'static DriverInfo;

    fn matches(&self, adv: &Advertisement<'_>) -> Option<DriverMatch>;

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
        nominal: FAMILY,
    };

    const FAMILY: crate::Nominal = crate::Nominal {
        limits: crate::NominalLimits {
            bolus: Some(crate::NominalBounds {
                min: crate::InsulinAmount::from_milliunits(50),
                max: crate::InsulinAmount::from_milliunits(10_000),
                tiers: &[crate::Tier {
                    from: crate::InsulinAmount::ZERO,
                    step: crate::InsulinAmount::from_milliunits(50),
                }],
            }),
            basal: None,
            temp_basal_percent: None,
            temp_basal_absolute: None,
            temp_basal_duration: None,
            extended_bolus_duration: None,
        },
        capabilities: crate::NominalCapabilities {
            reservoir_capacity: None,
            max_profile_segments: Some(24),
            min_segment_duration: None,
            history: true,
            daily_totals: false,
            patch: false,
        },
    };

    static SMALL: crate::Nominal = crate::Nominal {
        capabilities: crate::NominalCapabilities {
            reservoir_capacity: Some(crate::InsulinAmount::from_milliunits(100_000)),
            ..FAMILY.capabilities
        },
        ..FAMILY
    };

    #[test]
    fn a_model_without_figures_of_its_own_inherits_the_family() {
        let plain = Variant {
            id: "plain",
            name: "Plain",
            nominal: None,
        };
        let small = Variant {
            id: "small",
            name: "Small",
            nominal: Some(&SMALL),
        };
        assert_eq!(INFO.nominal_for(&plain), &FAMILY);
        assert_eq!(
            INFO.nominal_for(&small).capabilities.reservoir_capacity,
            Some(crate::InsulinAmount::from_milliunits(100_000))
        );
        assert_eq!(
            INFO.nominal_for(&small).limits,
            INFO.nominal_for(&plain).limits
        );
    }

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

    #[test]
    fn a_driver_match_carries_the_id_variant_and_derived_address() {
        const VARIANT: Variant = Variant {
            id: "v1",
            name: "V1",
            nominal: None,
        };
        let address = Address::new([0xEC, 0x2A, 0xF0, 1, 2, 3]);
        let m = DriverMatch {
            driver_id: INFO.id,
            variant: &VARIANT,
            address: Some(address),
        };
        assert_eq!(m.driver_id, "test");
        assert_eq!(m.variant.id, "v1");
        assert_eq!(m.address, Some(address));
        let same = m;
        assert_eq!(same, m);
        assert_ne!(DriverMatch { address: None, ..m }, m);
        assert!(alloc::format!("{m:?}").contains("DriverMatch"));
    }
}
