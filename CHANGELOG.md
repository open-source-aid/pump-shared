# Changelog

Entries separate **device-visible behaviour changes** (what a pump will now
receive differently) from **API changes**, as [chapter 12 of the
guideline](https://github.com/open-source-aid/guidelines/blob/main/12-versioning-and-releases.md)
requires.

## Unreleased

**Contract version: `API_VERSION` 1 → 2.**

### API changes

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
