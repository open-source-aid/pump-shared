# Changelog

Entries separate **device-visible behaviour changes** (what a pump will now
receive differently) from **API changes**, as [chapter 12 of the
guideline](https://github.com/open-source-aid/guidelines/blob/main/12-versioning-and-releases.md)
requires.

## Unreleased

**Contract version: `API_VERSION` 1 → 3** — two bumps, neither released yet:
1 → 2 for the public-key action, 2 → 3 for the nominal declaration below.

### API changes (`API_VERSION` 2 → 3)

- Added `DriverInfo::nominal: Nominal`, and `Variant::nominal:
  Option<&'static Nominal>` for a family whose models differ. A driver now
  declares what its device family is built for as a compile-time constant, so
  an app can draw a setup screen (supported bolus volumes, basal rates,
  profile segment count, reservoir capacity) before any device is connected.
  `DriverInfo::nominal_for(variant)` resolves the model's declaration against
  the family's.
- Added `Nominal`, `NominalLimits`, `NominalCapabilities` and
  `NominalBounds<T>`. They are deliberately *not* `Limits`, `Capabilities`
  and `Bounds`: nothing converts between the two sets, and `NominalBounds`
  has no `allows`, `floor` or `nearest`, only `values()` for a picker. A
  nominal figure describes a family and a device is narrower than its family,
  so validating or rounding a real dose against one would admit a command the
  device refuses. Every measurement in the nominal types is `Option`: a
  driver declares what it can evidence and leaves the rest absent.
- `DriverInfo` and `Variant` are structs every host reads, so both additions
  are breaking; `API_VERSION` is bumped to 3. Drivers that declare
  `api_version: pump_shared::API_VERSION` follow automatically.

### API changes (`API_VERSION` 1 → 2)

- Added `Action::PublicKey { id, key }`: the engine asks the host for the
  public half of a key the host holds (`KeyRef`), for protocols that must send
  that public half to the device.
- Added `Completion::PublicKey([u8; 32])`, its answer. It is plain bytes and
  not a `Secret`: a public key goes on the wire, so it needs neither zeroising
  nor a redacted `Debug`.
- `Action` and `Completion` are deliberately exhaustive, so both additions are
  breaking for every host; `API_VERSION` is bumped to 2 and hosts must handle
  the new variants. Drivers that declare `api_version: pump_shared::API_VERSION`
  follow automatically.
- Fixed `cargo test --no-default-features`: three test modules used
  `ToString` without importing it from `alloc`.

### Device-visible behaviour changes

- None. This release adds no protocol, handshake or device behaviour.
