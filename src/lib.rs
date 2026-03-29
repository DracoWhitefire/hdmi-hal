//! Hardware abstraction traits for the HDMI stack.
//!
//! `hdmi-hal` defines the behavioral contracts between protocol logic and hardware.
//! It is a traits-only crate: no implementations live here.

#![no_std]
#![forbid(unsafe_code)]

/// PHY lane configuration traits and associated types.
pub mod phy;

/// SCDC register transport trait.
pub mod scdc;
