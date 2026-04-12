# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-04-12

### Breaking changes

- **`ScdcTransport::read` now takes `&self` instead of `&mut self`.** Implementations
  that previously declared `fn read(&mut self, reg: u8)` must change the receiver to
  `&self`. Implementations that mutated internal state during reads (e.g. operation
  counters, offset tracking) must introduce interior mutability (`Cell`, `Mutex`, etc.)
  for those fields. The `write` method is unchanged and still takes `&mut self`.

  *Motivation:* register reads are logically non-mutating. The `&mut self` receiver
  was an implementation leak from `std::fs::File` requiring `&mut` for I/O, not a
  semantic requirement of the trait. Changing to `&self` allows transport references
  to be shared across concurrent read operations without a `Mutex` wrapper.

### Added

- **SLSA Build Level 2 provenance** — release artifacts are attested via
  `actions/attest-build-provenance` and verified with
  `gh attestation verify <file> --repo DracoWhitefire/hdmi-hal`.

## [0.3.0] - 2026-04-03

### Breaking changes

- **`EqParams` now has per-lane fields**: `lane0`, `lane1`, `lane2` (`LaneEqParams`)
  and `lane3` (`Option<LaneEqParams>`, `None` in 3-lane FRL mode). Implementations of
  `HdmiPhy::adjust_equalization` that previously ignored the empty struct should be
  updated to read the per-lane fields. `EqParams::new()` and `EqParams::default()`
  construct a valid zero-valued instance and are unchanged.

### Added

- `LaneEqParams` — per-lane equalization parameter struct, `#[non_exhaustive]`.
  Fields will be defined as the link training layer is implemented.
- `EqParams` and `LaneEqParams` now derive `Debug`, `Clone`, and `Copy`.

## [0.2.0] - 2026-04-03

### Breaking changes

- **`HdmiPhy::send_ltp` is a new required method**: existing `HdmiPhy` implementations
  must add `send_ltp(pattern: LtpPattern) -> Result<(), Self::Error>`. The method drives
  the link training pattern requested by the sink on the physical lanes during FRL
  training. The raw LFSR index (1 = LFSR0, 2 = LFSR1, 3 = LFSR2, 4 = LFSR3) is
  available via `LtpPattern::value()`.

### Added

- `LtpPattern` — newtype carrying a link training pattern index to be driven on the
  physical lanes. Constructed by link training state machines via `LtpPattern::new(u8)`;
  consumed by PHY backends via `LtpPattern::value() -> u8`. Raw value matches the SCDC
  Status_Flags encoding: 1 = LFSR0, 2 = LFSR1, 3 = LFSR2, 4 = LFSR3.
- `HdmiPhy::send_ltp(pattern: LtpPattern)` — drives the requested link training pattern
  on the physical lanes. Called by the FRL training loop on each iteration where the sink
  requests a non-zero pattern.

### Internal

- Unit tests for `HdmiPhy` and `EqParams` via a `MockPhy` implementation in `phy::tests`.
- Coverage ratchet CI job: measures line coverage with `cargo-llvm-cov`, checks against
  `.coverage-baseline`, and opens an automatic PR to advance the baseline when coverage improves.
- Fixed `.coverage-baseline`: value had been written with a locale comma separator
  (`100,00`); corrected to `100.00`.
- Fixed coverage ratchet CI: added `LC_NUMERIC=C` to the baseline `printf` to prevent
  locale-dependent decimal separators from corrupting `.coverage-baseline` on non-C locales.

## [0.1.0] - 2026-03-29

### Added

- `ScdcTransport` — raw register read/write access to the SCDC register map over
  DDC/I²C. Single-byte `read(reg: u8) -> Result<u8, Self::Error>` and
  `write(reg: u8, value: u8) -> Result<(), Self::Error>`. Associated `Error` type
  bounded by the implementing crate.
- `HdmiPhy` — PHY lane configuration for an HDMI 2.1 transmitter or receiver.
  Methods: `set_frl_rate(HdmiForumFrl)`, `adjust_equalization(EqParams)`, and
  `set_scrambling(bool)`. Associated `Error` type bounded by the implementing crate.
- `EqParams` — placeholder struct carrying equalization parameters passed from link
  training feedback to the PHY. Fields to be defined as the link training layer is
  implemented. Marked `#[non_exhaustive]`.
- `simulate` example — worked implementation of both traits against an in-memory
  register array, demonstrating the backend and consumer patterns downstream crates
  will follow.

**Project infrastructure**

- `#![no_std]`, `#![forbid(unsafe_code)]`
- `#[non_exhaustive]` on all public structs
- Full rustdoc coverage enforced via `cargo rustdoc -- -D missing_docs`
- CI workflow: fmt, clippy, docs
- Publish workflow: triggered on version tags, gated to commits reachable from `main`
