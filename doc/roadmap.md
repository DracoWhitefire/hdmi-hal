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

### Further PHY configuration

`HdmiPhy` covers what FRL link training needs. The mode-setting layer and platform
bring-up will also need:

- **Lane mapping** — lane order and polarity. Fixed by the board, so it may belong in a
  backend's configuration at construction rather than in the trait.
- **TMDS pre-emphasis and swing** — FRL's transmitter equalization is already covered by
  each lane's TxFFE level in `EqParams`; TMDS has no equivalent yet.
- **Spread-spectrum clocking.**
- **Power states** — powering the PHY down and up, for example across hotplug.

Each is decided when the layer that needs it is built. As trait methods with default
implementations they are not breaking changes; as required methods they would be.

Related: the compliance tester can restrict TxFFE to pre-shoot only, de-emphasis only or
no FFE (`Source_Test_Configuration`). If the link training crate honours these
restrictions rather than leaving them to the caller (an open question in plumbob), the PHY
may need them as further per-lane settings in `LaneEqParams`, which is `#[non_exhaustive]`
so they can be added without a breaking change.

### CEC line trait

A single-wire CEC bus access primitive: bit-bang read, write, and collision detection.
To be defined when CEC implementation begins. The bar for inclusion here is that both
the CEC protocol crate and at least one platform backend need to share the contract.
