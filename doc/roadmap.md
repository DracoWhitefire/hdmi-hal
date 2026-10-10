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

### CEC line trait

A single-wire CEC bus access primitive: bit-bang read, write, and collision detection.
To be defined when CEC implementation begins. The bar for inclusion here is that both
the CEC protocol crate and at least one platform backend need to share the contract.
