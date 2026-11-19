# Data model

The types every driver reports and every app reads. Drivers convert their pump's own units at the boundary; apps never see a pump-specific number.

## Principles

- **Integers in the smallest unit, no floating point.** Doses are never rounded silently.
- **One shared model, extended per pump.** Every report keeps the pump's raw bytes; a driver crate may offer a typed decoder for its own details.
- **Unknown is `None`.** A field a pump cannot report is absent, never guessed.
- **Data enums are `#[non_exhaustive]`.** Apps keep compiling when a pump adds an event. Host-contract enums (`Action`, `Completion`, `BleOp`) are exhaustive on purpose: a new action must be handled.

## Units

| Quantity | Type | Unit | Example |
|---|---|---|---|
| Insulin amount | `InsulinAmount` | milli-unit (0.001 U), `u32` | `1_525` = 1.525 U |
| Basal rate | `BasalRate` | milli-unit per hour, `u32` | `850` = 0.850 U/h |
| Relative temp basal | `Percent` | percent, `u16`, 100 = neutral | `150` = 150 % |
| Temp basal | `TempBasalRate` | `Percent(Percent)` or `Absolute(BasalRate)` | |
| Duration | `core::time::Duration` | whole seconds are used for limits | |
| Instant | `UtcTime` | Unix milliseconds, UTC | |
| Monotonic clock | `Timestamp` | host monotonic milliseconds | |
| Pump clock | `PumpTime` | pump-local seconds, as the pump shows them | |

Amounts are in the pump's own units (U100 volume). Insulin concentration is not part of the library: converting to and from the insulin actually delivered belongs to the caller.

## Converting

| Direction | Conversion |
|---|---|
| To floating-point units | `milliunits as f64 / 1000.0` |
| From floating-point units | `round(units * 1000.0)`, never truncation (`0.15 / 0.05` is `2.9999…`) |
| Absolute to percent temp basal | `BasalRate::percent_of(scheduled)`, nearest percent |
| Percent to absolute temp basal | `TempBasalRate::absolute(scheduled)`, rounded down |

A pump that only accepts one kind of temp basal converts the other with the scheduled rate from the active `BasalProfile`.

## Limits and rounding

`Limits` holds what the pump accepts, read from the pump when possible.

| Field | Type |
|---|---|
| `bolus` | `Bounds<InsulinAmount>` |
| `basal` | `Bounds<BasalRate>` |
| `temp_basal_percent` | `Option<Bounds<Percent>>` (`None`: not supported) |
| `temp_basal_absolute` | `Option<Bounds<BasalRate>>` |
| `temp_basal_duration` | `Bounds<Duration>` |
| `extended_bolus_duration` | `Option<Bounds<Duration>>` |

`Bounds<T>` is `min`, `max` and a list of `Tier { from, step }`: the step that applies to a value is the one of the last tier whose `from` is at or below it. Steps count from zero. Example, YpsoPump basal: 0.01 U/h from 0, 0.02 from 1, 0.1 from 2, 0.5 from 15.

| Method | Meaning |
|---|---|
| `allows(v)` | in range and on the grid |
| `floor(v)` | largest allowed value at or below `v` |
| `nearest(v)` | closest allowed value, ties round down |
| `values()` | every allowed value, ascending |

A driver refuses a value that `allows` rejects (`Refusal::OutOfLimits`); it never rounds a dose itself.

`Capabilities` adds what does not fit a range: reservoir capacity, maximum profile segments, minimum segment duration, whether the pump keeps history and daily totals, and whether it is a patch pump. `DeviceInfo` carries both, with serial, firmware and variant.

## Status

`PumpStatus` is a snapshot:

| Field | Meaning |
|---|---|
| `therapy` | `TherapyState`: mode, basal, bolus and extended bolus regions (see [`therapy.mmd`](therapy.mmd) and [`therapy-overview.mmd`](therapy-overview.mmd)) |
| `battery` | `Option<Percent>` |
| `reservoir` | `Option<InsulinAmount>` |
| `basal_rate` | the rate running now |
| `clock` | `Option<PumpTime>`; `to_utc(offset)` and `drift(offset, now)` |
| `delivery_uncertain` | a command may or may not have been acted on |
| `alarms` | active `Alarm`s |

## Basal profile

`BasalProfile` is a list of `BasalSegment { start, rate }`, validated: at least one segment, the first at midnight, starts strictly increasing, none at or after 24 h. `rate_at(time_since_midnight)` returns the rate in force.

## History

Each `HistoryEntry` holds `sequence` (the pump's own number), `time` (UTC), `pump_time` (as the pump recorded it), `event` and `raw` (the bytes as read). `sync_id(serial)` returns `"<serial>-<sequence>"`, stable across reads, for deduplication.

| Event | Fields |
|---|---|
| `Bolus` | `kind` (`Normal`, `Smb`, `Priming`, `Extended`, `Combined`), `requested`, `delivered`, `duration`, `automatic`, `in_progress` |
| `ExtendedBolusEnded` | `delivered` |
| `TempBasal` | `rate`, `duration`, `phase` (`TempBasalPhase`: `Started`, `Running`, `Ended`) |
| `Suspended` / `Resumed` | |
| `BasalProfileChanged` | |
| `Rewound` | |
| `Replaced` | `Component`: `Reservoir`, `Cannula`, `Tubing`, `Battery`, `Pump` |
| `ClockChanged` | `previous` pump time |
| `Alarm` / `AlarmCleared` | `Alarm { kind, code }`: normalised kind plus the pump's raw code |
| `DailyTotal` | `day`, `amount` |
| `LimitChanged` | `LimitChange`: `MaxBolus { value, previous }` (`InsulinAmount`), `MaxBasal { value, previous }` (`BasalRate`), `BolusStep { value }` (`InsulinAmount`) |

Alarm kinds: `Occlusion`, `ReservoirLow`, `ReservoirEmpty`, `BatteryLow`, `BatteryEmpty`, `AutoOff`, `NoDelivery`, `Other`.

## Commands and results

| Command | Delivers insulin | Stop path |
|---|---|---|
| `Status`, `ReadClock`, `ReadHistory { after }`, `ReadBasalProfile` | no | |
| `Pair`, `Unpair`, `SyncClock`, `SetDeliveryLimits { max_bolus, max_basal }` | no | |
| `Bolus { amount }`, `ExtendedBolus { amount, duration }` | yes | |
| `RateBolus { amount, duration }` | yes | |
| `TempBasal { rate, duration }` | yes, unless 100 % | 100 % is |
| `SetBasalProfile { profile }`, `Resume` | yes | |
| `CancelTempBasal`, `CancelExtendedBolus`, `Suspend` | no | yes |

Stop paths are never refused for a missing feature or a pump state. `Resume` is only accepted from a suspended pump.

A finished command reports an `Outcome`: `Completed(Report)`, `Approximated(Approximation)` (carried out, but not exactly as requested), `Refused(Refusal)` (nothing was sent), `Unknown` (something was sent and the result is not known; never retried) or `Failed(Failure)`. `Report` is `Done`, `Status(PumpStatus)`, `Clock(PumpTime)`, `Delivered(InsulinAmount)`, `Profile(BasalProfile)` or `History(Vec<HistoryEntry>)`.

`Approximation` carries both what was requested and what was programmed, so the app sees the direction and size of the gap: `Bolus { requested, delivered, duration }` (both `InsulinAmount`) for a rate-controlled bolus, and `Basal { requested, achieved }` (both `BasalRate`) for an absolute temp basal.

`RateBolus { amount, duration }` asks for a total `amount` spread over `duration` at a controlled rate, delivered as a sequence of small immediate boluses. It completes with `Delivered` when the whole total lands exactly, and with `Approximated(Bolus { requested, delivered, duration })` when the sequence delivers a different total (for example a request that is not a multiple of the bolus step, or a sequence cut short).

## Integrating

Per-app differences the app-side adapter must handle: [AndroidAPS](integration/aaps.md), [Loop](integration/loop.md), [Trio](integration/trio.md).
