//! What a device *family* is built for, declared by the driver at compile
//! time — as opposed to what a particular connected device reports.
//!
//! # Nominal is not actual
//!
//! [`Limits`](crate::Limits) and [`Capabilities`](crate::Capabilities) are
//! read **from a device**, and reach a host inside
//! [`DeviceInfo`](crate::DeviceInfo), that is, only after a connect. An app
//! has to draw a setup screen before that: which bolus volumes to offer, how
//! many basal segments a profile may have, how big the reservoir is. The
//! types in this module let a driver answer that question up front, from a
//! `const`.
//!
//! Everything here is *nominal*: the widest grid the family is built to
//! accept, the ceiling no model of it will exceed, the capacity of the
//! cartridge the family takes. A particular pump is **narrower** than its
//! family — its bolus step is a user setting, its maximum bolus is a user
//! setting, its firmware may not have extended bolus at all — and only the
//! device itself can say by how much.
//!
//! So a nominal figure is good for one thing: **offering a choice before a
//! device exists**. It is never a validation input. Validating a real dose
//! against a nominal ceiling would let through a command the device will
//! refuse — and a nominal *step* would be worse, because the device would
//! round it, or take it, and the app would have logged something else.
//!
//! Three things keep the two apart, by construction:
//!
//! 1. **Separate types.** [`NominalLimits`] is not [`Limits`](crate::Limits),
//!    [`NominalBounds`] is not [`Bounds`](crate::Bounds). Nothing converts
//!    between them: there is no `From`, no `to_limits()`, no `to_bounds()`.
//!    A nominal value cannot reach
//!    [`Limits::allows_temp_basal`](crate::Limits::allows_temp_basal) or any
//!    other check, in any host language, even by mistake.
//! 2. **No verbs that decide.** [`NominalBounds`] has no `allows`, no
//!    `floor`, no `nearest`. It offers [`values()`](NominalBounds::values),
//!    which enumerates a picker, and its `min`/`max`/`tiers` to read. The
//!    verbs that gate or round a dose exist only on
//!    [`Bounds`](crate::Bounds), which only a device can fill in.
//! 3. **The name travels with the value.** Every type here is `Nominal*`, and
//!    it is reached through [`DriverInfo::nominal`](crate::DriverInfo), never
//!    through `DeviceInfo`. A reader of `info.nominal.limits.bolus.max` is
//!    told what it is at every step of the path.
//!
//! A driver declares only what it can evidence. A figure nobody has measured
//! is absent (`None`), never guessed, as everywhere else in this contract.

use alloc::vec::Vec;
use core::time::Duration;

use crate::device::limits::step_at;
use crate::{BasalRate, InsulinAmount, Percent, Quantity, Tier};

/// Every value on a tier grid between `min` and `max`, ascending.
///
/// [`NominalBounds::values`] is this, and so is
/// [`Bounds::values`](crate::Bounds::values); it is public because a
/// `NominalBounds` that has crossed an ABI no longer has its `'static` tiers
/// and cannot be rebuilt, so a binding layer enumerates it from the borrowed
/// table instead of growing an enumeration of its own.
///
/// Enumerating is not deciding: this says what a grid holds, never whether a
/// device accepts a dose.
pub fn grid<T: Quantity>(min: T, max: T, tiers: &[Tier<T>]) -> Vec<T> {
    let mut values = Vec::new();
    let mut raw = min.raw();
    while raw <= max.raw() {
        let Some(step) = step_at(tiers, raw) else {
            break;
        };
        if raw % step == 0 {
            values.push(T::from_raw(raw));
            raw += step;
        } else {
            raw += step - raw % step;
        }
    }
    values
}

/// A grid the *family* is built to accept, for a picker on a setup screen.
///
/// The same `min`/`max`/`tiers` shape as [`Bounds`](crate::Bounds), so a host
/// reads one tier table, but `'static` rather than owned (a driver declares
/// it in a `const`) and without the verbs that decide a dose. See the
/// [module docs](self) for why those are missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NominalBounds<T: 'static> {
    /// The smallest value any model of the family can be set to.
    pub min: T,
    /// The largest value any model of the family can be set to, before the
    /// user's own cap on the device narrows it.
    pub max: T,
    /// The finest grid the family offers. A device may use a coarser one.
    pub tiers: &'static [Tier<T>],
}

impl<T: Quantity> NominalBounds<T> {
    /// Every value on the nominal grid, ascending — the list to populate a
    /// setup picker with.
    ///
    /// Not a validation list: a connected device accepts a subset of it, and
    /// [`Bounds`](crate::Bounds) from that device is what says which.
    pub fn values(&self) -> Vec<T> {
        grid(self.min, self.max, self.tiers)
    }
}

/// Nominal counterpart of [`Limits`](crate::Limits).
///
/// Every field is `Option`: a driver that cannot evidence a figure leaves it
/// absent rather than inventing one, and a host that finds `None` asks the
/// user or waits for the device instead. This is the one shape difference
/// from `Limits`, where `bolus`, `basal` and `temp_basal_duration` are
/// mandatory — a connected device always knows them; a driver author writing
/// a constant may not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NominalLimits {
    /// Bolus amounts. `max` is the family's ceiling, not a user's maximum
    /// bolus setting, which is lower and lives on the device.
    pub bolus: Option<NominalBounds<InsulinAmount>>,
    /// Basal rates the family can be programmed with.
    pub basal: Option<NominalBounds<BasalRate>>,
    /// Relative temp basal, `None` when the family has none.
    pub temp_basal_percent: Option<NominalBounds<Percent>>,
    /// Absolute temp basal, `None` when the family has none.
    pub temp_basal_absolute: Option<NominalBounds<BasalRate>>,
    /// How long a temp basal may run.
    pub temp_basal_duration: Option<NominalBounds<Duration>>,
    /// How long the family's own extended bolus may run, `None` when the
    /// family has no extended bolus.
    pub extended_bolus_duration: Option<NominalBounds<Duration>>,
}

/// Nominal counterpart of [`Capabilities`](crate::Capabilities).
///
/// The three flags are family facts a driver author knows without a device
/// (a patch pump family is a patch pump family). The three measurements are
/// `Option`, for the same reason as in [`NominalLimits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NominalCapabilities {
    /// The capacity of a full cartridge or reservoir, not the fill level.
    pub reservoir_capacity: Option<InsulinAmount>,
    /// How many segments a basal profile may hold.
    pub max_profile_segments: Option<u8>,
    /// The shortest basal profile segment.
    pub min_segment_duration: Option<Duration>,
    /// The family keeps a readable history log.
    pub history: bool,
    /// The family keeps daily totals.
    pub daily_totals: bool,
    /// The family is a patch pump.
    pub patch: bool,
}

/// Everything a driver can say about its family before a device is connected.
///
/// Hangs on [`DriverInfo::nominal`](crate::DriverInfo) for the family, and
/// may be overridden per model by
/// [`Variant::nominal`](crate::Variant) when models differ — a bigger
/// reservoir, a lower bolus ceiling. [`DriverInfo::nominal_for`] picks the
/// right one, and a host that has only matched an advertisement already has
/// the [`Variant`](crate::Variant) to pass it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Nominal {
    pub limits: NominalLimits,
    pub capabilities: NominalCapabilities,
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: Tier<InsulinAmount> = Tier {
        from: InsulinAmount::ZERO,
        step: InsulinAmount::from_milliunits(50),
    };

    const BOLUS: NominalBounds<InsulinAmount> = NominalBounds {
        min: InsulinAmount::from_milliunits(50),
        max: InsulinAmount::from_milliunits(200),
        tiers: &[STEP],
    };

    #[test]
    fn enumerates_the_family_grid_for_a_picker() {
        assert_eq!(
            BOLUS.values(),
            [50, 100, 150, 200].map(InsulinAmount::from_milliunits)
        );
    }

    #[test]
    fn an_undeclared_family_says_nothing() {
        let nominal = Nominal::default();
        assert_eq!(nominal.limits.bolus, None);
        assert_eq!(nominal.capabilities.reservoir_capacity, None);
        assert!(!nominal.capabilities.patch);
        assert_eq!(nominal, nominal.clone());
        assert!(alloc::format!("{nominal:?}").contains("Nominal"));
    }

    #[test]
    fn carries_bounds_and_capabilities_separately() {
        let nominal = Nominal {
            limits: NominalLimits {
                bolus: Some(BOLUS),
                ..NominalLimits::default()
            },
            capabilities: NominalCapabilities {
                max_profile_segments: Some(24),
                min_segment_duration: Some(Duration::from_secs(3600)),
                history: true,
                ..NominalCapabilities::default()
            },
        };
        assert_eq!(nominal.limits.bolus.unwrap().max.milliunits(), 200);
        assert_eq!(nominal.capabilities.max_profile_segments, Some(24));
        assert_ne!(nominal, Nominal::default());
    }
}
