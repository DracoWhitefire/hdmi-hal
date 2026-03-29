# hdmi-hal

Hardware abstraction traits for the HDMI stack.

`hdmi-hal` defines the behavioral contracts between protocol logic and hardware. It is a
traits-only crate: no implementations live here. Every trait expresses an I/O boundary
that multiple crates in the stack need to cross in a compatible way.

## Traits

- [`ScdcTransport`] — raw register read/write access to the SCDC register map over DDC/I²C
- [`HdmiPhy`] — PHY lane configuration: FRL rate selection, equalization, scrambling

## `no_std`

This crate is `no_std` compatible. No trait method requires allocation.

## License

Mozilla Public License 2.0
