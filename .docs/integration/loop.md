# Loop (LoopKit)

What differs between pump-shared's data and LoopKit's `PumpManager`, and what the app-side adapter must convert.

| Topic | pump-shared | LoopKit | Adapter |
|---|---|---|---|
| Insulin, rates | `u32` milli-units, mU/h | `Double` U, U/h | out: `mU / 1000.0`; in: `round(u * 1000.0)` |
| Off-grid values | refused (`Refusal::OutOfLimits`) | rounded down to a supported value (`roundToSupportedBolusVolume`) | round with `Bounds::floor` before `submit` |
| Supported values | `Bounds::values()` | `supportedBolusVolumes`, `supportedBasalRates` | expose `values()` as is |
| Temp basal | `Percent` or `Absolute` | absolute only (`enactTempBasal(unitsPerHour:)`) | percent-only pumps: `BasalRate::percent_of(scheduled)` from the active profile |
| Cancel temp basal | `Command::CancelTempBasal` | `enactTempBasal` with duration 0 | map duration 0 to `CancelTempBasal` |
| Extended bolus | supported | not supported | do not expose |
| Battery | `Option<Percent>` | `pumpBatteryChargeRemaining`, 0 to 1 | `percent / 100.0` |
| Reservoir | in `PumpStatus` | separate callback, `didReadReservoirValue` | send it on each status |
| Uncertain result | `Outcome::Unknown` | `PumpManagerError.uncertainDelivery`, `deliveryIsUncertain` | set both, never retry |
| History id | `sync_id` | `NewPumpEvent.raw` as `syncIdentifier` | `raw` = `sync_id` bytes |
| Bolus in progress | `Bolus { in_progress: true }` | `DoseEntry.isMutable` | map directly |
| Alarms | `AlarmKind` | `PumpAlarmType` | `ReservoirEmpty` = `noInsulin`, `BatteryEmpty` = `noPower`, `BatteryLow` = `lowPower`, `ReservoirLow` = `lowInsulin`, `Other` = `other(code)` |
| Limits | `Command::SetDeliveryLimits` | `syncDeliveryLimits` | map directly |
| Profile limits | `Capabilities.max_profile_segments`, `min_segment_duration` | `maximumBasalScheduleEntryCount`, `minimumBasalScheduleEntryDuration` | map directly |
| Time | `UtcTime` ms | `Date` | `Date(timeIntervalSince1970: ms / 1000)` |
