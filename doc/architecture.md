# Architecture

## Role

`hdmi-hal` defines the behavioral contracts between protocol logic and hardware across
the HDMI stack. It is a traits-only crate: no implementations live here. Every trait
in this crate expresses an I/O boundary that multiple crates in the stack need to cross
in a compatible way.

The bar for inclusion is that at least two crates must share the contract. Traits that
are only ever used by a single library stay in that library.

---

## Scope

`hdmi-hal` covers:

- `ScdcTransport` — raw register read/write access to the SCDC register map over DDC/I²C
- `HdmiPhy` — PHY lane configuration: FRL rate, equalization, scrambling
- CEC line access trait — when CEC is implemented

The following are explicitly out of scope:

- **Protocol logic** — SCDC register semantics, typed register wrappers, and the FRL
  link training state machine belong in the crates that consume `ScdcTransport`.
- **Implementations** — concrete platform backends (kernel I²C drivers, simulator
  harnesses, test doubles) live in the crates that use these traits. No implementation
  of any trait in this crate lives in this crate.
- **PHY vendor backends** — hardware-specific register sequences belong in platform
  crates that implement `HdmiPhy`.
- **CEC or eARC protocol logic** — only the wire-access primitive belongs here; the
  protocol state machines are in their own crates.

---

## Dependency Position

`hdmi-hal` depends on `display-types` for shared HDMI vocabulary types (`HdmiForumFrl`,
`FrlRate`). It does not depend on `piaf`, `concordance`, or any other stack crate.

```
display-types   ←   hdmi-hal        ←   culvert
                                    ←   plumbob
                                    ←   (CEC)
                                    ←   platform backends (sync)

display-types   ←   hdmi-hal-async  ←   culvert-async
hdmi-hal        ↗                   ←   plumbob-async
                                    ←   platform backends (Embassy / async)
```

`display-types` does not depend on `hdmi-hal`. The two crates share no circular
dependency; `display-types` defines vocabulary, `hdmi-hal` defines contracts.

`hdmi-hal-async` is a companion crate that mirrors each trait in `hdmi-hal` with
`async fn` methods and identical signatures otherwise. It depends on `hdmi-hal` to
reuse shared data types (`EqParams` and any future structs) rather than duplicating
them. Implementors target one or both crates depending on their executor environment;
consumers of both sync and async backends accept the relevant trait bound.

---

## Traits

### `ScdcTransport`

Abstracts raw register access to the SCDC register map over DDC/I²C.

SCDC is a bidirectional channel used for FRL link training initiation, scrambling
control, and CED (Character Error Detection) reporting. Any code that needs to touch
the SCDC register map does so through this trait.

```rust
pub trait ScdcTransport {
    type Error;

    fn read(&mut self, reg: u8) -> Result<u8, Self::Error>;
    fn write(&mut self, reg: u8, value: u8) -> Result<(), Self::Error>;
}
```

The trait operates at the raw register level: a one-byte address and a one-byte value.
The typed SCDC register wrappers — named constants, bitfield structs, multi-register
sequences — belong in the `culvert` crate, not here. Keeping this boundary clean means
`hdmi-hal` encodes no knowledge of the SCDC specification; it only describes how bytes
move.

The associated `Error` type is bounded by the implementing crate. A hardware backend
exposes its own I²C error type; a simulator may use `Infallible`.

**Block read/write.** Some SCDC operations benefit from burst reads (e.g. reading all
CED registers in a single transaction). A default-provided multi-byte variant may be
added here if it surfaces as a consistent need across consumers; for now, single-register
access is the defined contract.

---

### `HdmiPhy`

Abstracts the register sequences required to configure an HDMI 2.1 PHY: lane mapping,
pre-emphasis, equalization, scrambling, and FRL rate selection.

```rust
pub trait HdmiPhy {
    type Error;

    /// Select the FRL rate (or TMDS). Triggers the required lane reconfiguration sequence.
    fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error>;

    /// Drive a Link Training Pattern on the PHY lanes.
    fn send_ltp(&mut self, pattern: LtpPattern) -> Result<(), Self::Error>;

    /// Adjust equalization parameters after link training feedback.
    fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error>;

    /// Enable or disable scrambling on the PHY.
    fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error>;
}
```

`HdmiForumFrl` is from `display-types`. `LtpPattern` is a newtype defined in this crate
wrapping the raw pattern index from the SCDC Status_Flags register (1–4 for LFSR0–LFSR3,
0 for the exit condition). This keeps `hdmi-hal` free of any dependency on `plumbob`;
the link training crate converts from its own `LtpReq` type to `LtpPattern` before
calling the PHY. `EqParams` carries per-lane equalization data derived from CED feedback during the FRL
training loop: `lane0`, `lane1`, `lane2` (`LaneEqParams`) and `lane3`
(`Option<LaneEqParams>`, `None` in 3-lane FRL mode). The fields of `LaneEqParams` will
be defined as the link training state machine is implemented and the actual per-lane
equalization knobs are known.

Like `ScdcTransport`, implementations are entirely in platform crates. The trait surface
is driven by what the link training and mode-setting layers need to call; vendor-specific
register sequences are an implementation detail.

---

### CEC Line Trait _(planned)_

Will abstract single-wire CEC bus access: bit-bang read, write, and collision detection.
Protocol state machines (device discovery, logical address allocation, command dispatch)
are not here; only the wire primitive. Will be defined when CEC implementation begins.

---

## `no_std` Compatibility

All traits in this crate must be usable in bare `no_std` environments. This means:

- No trait method may require allocation.
- `Error` associated types may be `Infallible` in no-alloc implementations.
- No default implementations may bring in `std` dependencies.

---

## Async Compatibility

`hdmi-hal` defines synchronous traits. Async variants — identical in shape, with
`async fn` methods — live in the companion crate `hdmi-hal-async`, following the same
split as `embedded-hal` / `embedded-hal-async`.

The rules that make this work cleanly:

- **Trait shapes are identical.** `hdmi-hal-async::ScdcTransport` and
  `hdmi-hal-async::HdmiPhy` mirror their sync counterparts exactly; only method
  signatures gain `async`. No behavioral differences, no extra associated types.
- **Data types are shared.** `EqParams` and any other plain data structs are defined
  in `hdmi-hal` and re-exported or referenced from `hdmi-hal-async`. There is one
  definition of each type regardless of which trait variant the caller uses.
- **Implementors choose one or both.** A bare-metal driver implements `hdmi-hal`
  traits. An Embassy driver implements `hdmi-hal-async` traits. A simulator can
  implement both. No crate is required to support both.
- **Consumer crates are generic over the trait bound.** A link training crate that
  needs async I/O takes `T: hdmi_hal_async::ScdcTransport`; one that is sync takes
  `T: hdmi_hal::ScdcTransport`. The HAL layer imposes no executor dependency on sync
  consumers.

`hdmi-hal-async` is out of scope for the current implementation phase. The trait
surfaces defined here are designed so that adding the async companion later requires
no changes to this crate.

---

## Simulated Implementations

`hdmi-hal` defines the contracts; it does not provide test doubles or simulators.
Each consuming crate is responsible for its own simulated transport. This is the same
pattern as piaf's handler abstraction: the library defines the interface and ships
tests against real (byte-slice) data; the integrator provides the platform backend.

A typical test harness for SCDC work:

```rust
struct SimulatedScdc {
    registers: [u8; 256],
}

impl ScdcTransport for SimulatedScdc {
    type Error = Infallible;

    fn read(&mut self, reg: u8) -> Result<u8, Infallible> {
        Ok(self.registers[reg as usize])
    }

    fn write(&mut self, reg: u8, value: u8) -> Result<(), Infallible> {
        self.registers[reg as usize] = value;
        Ok(())
    }
}
```

The link training state machine can be driven to any desired state by pre-loading the
simulated register array, without requiring real hardware or a kernel I²C driver.

---

## Design Principles

- **Contracts only.** No implementation of any trait lives in this crate. The moment an
  implementation appears here, it becomes a platform assumption baked into a foundational
  dependency.
- **Spec accuracy and completeness.** Trait surfaces must not omit operations the
  specification defines. A method left out because it seems unlikely to be needed now
  is a missing contract that forces a breaking change or a fork later. What callers
  choose to use is their decision; what the spec defines is the scope.
- **Stack-ordered delivery.** The 0.1.0 release scope covers what the next layer in the
  stack needs to function. Everything else the spec defines still gets built, but is
  tracked on the roadmap until the layer that needs it is being implemented. This is a
  sequencing decision, not a judgement about what is worth implementing.
- **Raw boundaries, typed protocols.** `ScdcTransport` is intentionally byte-level.
  The typed SCDC register map belongs in the protocol crate. Mixing them would encode
  HDMI specification knowledge into what should be a pure I/O contract.
- **Deterministic and testable.** Every consumer of a trait in this crate must be fully
  exercisable with a simulated implementation. Hardware is never a test dependency.
- **No unsafe code.** `#![forbid(unsafe_code)]` is a hard constraint. Hardware register
  access via unsafe is the implementing crate's concern, not this crate's.
- **Extensible without forking.** The trait is the extension point. A platform with a
  proprietary PHY, a non-standard I²C stack, or a simulated transport for testing
  implements the relevant trait and plugs in — no source modification required anywhere
  in the stack. This is the HAL layer's primary contribution to that stack-wide property.
- **Stable contracts.** Trait changes are breaking changes for every implementor. The bar
  for modifying a trait surface after initial publication is high; additions go through
  careful review of what all known consumers require.
- **Sync / async parity.** The sync traits in this crate and the async traits in
  `hdmi-hal-async` must stay in lockstep. A change to a trait surface here requires a
  corresponding change to its async mirror. This is enforced by convention, not by the
  type system.
- **Attested releases.** Every release is published through a GitHub Actions workflow
  that signs the `.crate` package with [SLSA Build Level 2](https://slsa.dev) provenance.
  Verify with `gh attestation verify <file> --repo DracoWhitefire/hdmi-hal`.
