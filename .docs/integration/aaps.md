# AndroidAPS

What differs between pump-shared's data and AndroidAPS's, and what the app-side adapter must convert.

| Topic | pump-shared | AndroidAPS | Adapter |
|---|---|---|---|
| Insulin, rates | `u32` milli-units, mU/h | `Double` U, U/h (`PumpInsulin`, `PumpRate`) | out: `mU / 1000.0`; in: `round(u * 1000.0)` |
| Off-grid values | refused (`Refusal::OutOfLimits`) | rounded to the nearest step (`PumpType.determineCorrectBolusSize`) | round with `Bounds::nearest` before `submit` |
| Temp basal | `TempBasalRate::Percent` or `Absolute` | `setTempBasalPercent` / `setTempBasalAbsolute`, style declared per pump | map directly; refuse the style `Limits` does not offer |
| Suspend | `Command::Suspend` / `Resume` | no command; emulated with a zero temp basal | AAPS sends a zero temp basal; `Suspend` stays unused |
| Bolus kinds | `Normal`, `Smb`, `Priming`, `Extended` | `BS.Type` `NORMAL`, `SMB`, `PRIMING`; extended bolus is an `EB` record | `Extended` events become `EB` records |
| History id | `sync_id` = `"<serial>-<sequence>"` | `pumpId: Long` plus `pumpSerial` (`IDs`) | `pumpId` = `sequence`, `pumpSerial` = serial |
| Temp basal end | `TempBasalEnded` event | `syncStopTemporaryBasalWithPumpId(endPumpId)` | the end event's `sequence` is `endPumpId` |
| Limits | `Limits`, `Capabilities`, known after connect | `PumpDescription`, set when the plugin starts | fill from defaults, update on `Event::DeviceInfo` |
| Step tiers | `Bounds.tiers` | `DoseStepSize` tables | build `DoseStepSize` from `tiers` |
| Uncertain result | `Outcome::Unknown` | `PumpEnactResult` with `success = false` | report as not enacted and never retry |
| Concentration | pump-native units | `PumpInsulin(cU)` converted by `PumpSync` | pass pump-native values; AAPS converts |
| Time | `UtcTime` ms, `PumpTime` local | epoch ms plus `utcOffset` per record | `utcOffset` = the offset used for `PumpTime::to_utc` |
