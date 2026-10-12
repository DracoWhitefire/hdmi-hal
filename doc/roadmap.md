# Roadmap

What each release contains is in the [changelog](../CHANGELOG.md); this page is about
what comes next.

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

### CEC line trait

A single-wire CEC bus access primitive: bit-bang read, write, and collision detection.
To be defined when CEC implementation begins. The bar for inclusion here is that both
the CEC protocol crate and at least one platform backend need to share the contract.
