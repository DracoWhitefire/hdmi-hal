# Roadmap

## Released

### 0.3.0

- `LaneEqParams` — per-lane equalization parameter struct, `#[non_exhaustive]`. Fields
  will be defined as the link training layer is implemented.
- `EqParams` now carries per-lane fields: `lane0`, `lane1`, `lane2` (`LaneEqParams`) and
  `lane3` (`Option<LaneEqParams>`, `None` in 3-lane FRL mode). **Breaking change** for
  `HdmiPhy::adjust_equalization` implementations that constructed `EqParams` directly;
  use `EqParams::new()` or `EqParams::default()` to get a valid zero-valued instance.
- `EqParams` and `LaneEqParams` now derive `Debug`, `Clone`, and `Copy`.

### 0.2.0

- `LtpPattern` — newtype carrying the raw link training pattern index from the SCDC
  Status_Flags register, passed to the PHY on each FRL training iteration.
- `HdmiPhy::send_ltp(pattern: LtpPattern)` — new required method; drives the requested
  pattern on the physical lanes during FRL training.

### 0.1.0

Trait surfaces covering what the SCDC and link training layers need to function:

- `ScdcTransport` — raw register read/write over DDC/I²C
- `HdmiPhy` — FRL rate selection, equalization adjustment, scrambling control
- `EqParams` — placeholder struct; fields defined as link training layer is implemented

## Planned

### Block read/write for `ScdcTransport`

Some SCDC operations benefit from burst reads — reading all CED registers in a single
I²C transaction rather than one byte at a time. A default-provided multi-byte variant
will be added once the SCDC or link training layer surfaces a consistent need for it.

### Full `LaneEqParams` fields

`LaneEqParams` ships with no fields in 0.3.0. Its fields will be defined once the link
training state machine is implemented and the actual per-lane equalization knobs are
known. `#[non_exhaustive]` is retained permanently on both `EqParams` and `LaneEqParams`:
it prevents external crates from using struct literal syntax, ensuring that new fields
can always be added without a breaking change.

### CEC line trait

A single-wire CEC bus access primitive: bit-bang read, write, and collision detection.
To be defined when CEC implementation begins. The bar for inclusion here is that both
the CEC protocol crate and at least one platform backend need to share the contract.

### `hdmi-hal-async`

Async companion crate mirroring each trait in `hdmi-hal` with `async fn` methods and
identical signatures otherwise, following the same split as `embedded-hal` /
`embedded-hal-async`. Shares data types (`EqParams` and any future structs) with
`hdmi-hal` rather than duplicating them. To be implemented when an async consumer
(Embassy platform backend, async link training) is being built.
