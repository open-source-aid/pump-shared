# Trio

Trio drives pumps through LoopKit's `PumpManager`: everything in [loop.md](loop.md) applies. Differences on top of it:

| Topic | pump-shared | Trio | Adapter |
|---|---|---|---|
| Stored amounts | `u32` milli-units | `Decimal` | `Decimal(mU) / 1000` is exact; never go through `Double` |
| Bolus increment | `Bounds.tiers` | `bolusIncrement`, taken from the first supported bolus volume | with tiered steps, the first tier's step only |
| Temp basal type | `Percent` or `Absolute` | stores every temp basal as absolute | convert percent with the active profile before storing |
| Durations | `Duration` | whole minutes | `secs / 60`; durations in `Limits` are whole minutes for supported pumps |
| Battery | `Option<Percent>` | `percent: Int?` | map directly |
