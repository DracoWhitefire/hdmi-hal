# Roadmap

## Planned

### 0.1.0 — Initial release

Trait surfaces covering what the SCDC and link training layers need to function:

- `ScdcTransport` — raw register read/write over DDC/I²C
- `HdmiPhy` — FRL rate selection, equalization adjustment, scrambling control
- `EqParams` — placeholder struct; fields defined as link training layer is implemented

### Block read/write for `ScdcTransport`

Some SCDC operations benefit from burst reads — reading all CED registers in a single
I²C transaction rather than one byte at a time. A default-provided multi-byte variant
will be added once the SCDC or link training layer surfaces a consistent need for it.

### Full `EqParams` fields

`EqParams` ships as a placeholder in 0.1.0. Its fields will be defined once the link
training state machine is implemented and the actual per-lane equalization knobs are
known. At that point the struct gets `#[non_exhaustive]` removed and the fields locked.

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
