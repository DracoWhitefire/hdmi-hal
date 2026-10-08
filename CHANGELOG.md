# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking changes

- **`LtpPattern` is now a `#[non_exhaustive]` enum with the HDMI 2.1 pattern values.**
  The previous documentation mapped 1–4 to LFSR 0–3, which was wrong: the values are
  1 = all ones, 2 = all zeros, 3 = Nyquist clock, 4 = DDE compliance and 5–8 = LFSR 0–3.
  `LtpPattern::new(u8)` is removed; use the variants (`LtpPattern::Lfsr0` etc.).
  `LtpPattern::value()` remains and is now `const`.
- **`HdmiPhy::send_ltp` takes the full per-lane pattern set**: the signature is now
  `send_ltp(patterns: LanePatterns)`. HDMI 2.1 sinks request a pattern per lane, so a
  single pattern for all lanes could not express training. `None` on a lane means no
  training pattern; `None` on every lane stops the training patterns.

### Added

- `LanePatterns` — the link training pattern for each lane (`lane0` to `lane3`, each an
  `Option<LtpPattern>`; `lane3` is `None` in 3-lane FRL mode). Derives `Default` (no
  pattern on any lane), `Debug`, `Clone`, `Copy`, `PartialEq` and `Eq`.
- `TxFfeLevel` — a TxFFE level index, 0–7. `TxFfeLevel::new(u8)` returns `None` above 7;
  `TxFfeLevel::MAX` is level 7 and the default is level 0. Read with `value()`.
- `LaneEqParams::tx_ffe_level` — the lane's TxFFE level, applied through
  `HdmiPhy::adjust_equalization`. Defaults to level 0.
- `EqParams` and `LaneEqParams` now derive `PartialEq` and `Eq`.

### Internal

- **Publish dispatches restricted to release tags** — `publish.yml` already accepted
  `workflow_dispatch` (used by `release-tag`), but a dispatch against a branch such as
  `main` would have published the version on that branch and created a GitHub release
  named after the branch. Dispatches against a non-tag ref are now skipped, so they cannot
  publish or create a release.

## [0.4.1] - 2026-05-20

### Changed

- **`display-types` dependency bumped from `0.3.1` to `0.4`.**

### Internal

- `examples/simulate/Cargo.lock` is no longer tracked in git and is now listed in
  `.gitignore`, and `--locked` has been dropped from the CI build of the simulate example.
  The simulate example is `publish = false`; committing its lock file provided no
  reproducibility benefit and required manual updates on every version bump.

## [0.4.0] - 2026-04-13

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
