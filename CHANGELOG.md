# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Internal

- Unit tests for `HdmiPhy` and `EqParams` via a `MockPhy` implementation in `phy::tests`.
- Coverage ratchet CI job: measures line coverage with `cargo-llvm-cov`, checks against
  `.coverage-baseline`, and opens an automatic PR to advance the baseline when coverage improves.

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
