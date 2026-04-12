//! Simulated backend example for `hdmi-hal`.
//!
//! Demonstrates the implementation pattern for [`ScdcTransport`] and [`HdmiPhy`]:
//! a concrete backend backed by an in-memory register array, and consumer functions
//! generic over the trait — the signature pattern all downstream crates will follow.
//!
//! Run with `cargo run` from this directory.

use core::convert::Infallible;
use display_types::cea861::hdmi_forum::HdmiForumFrl;
use hdmi_hal::phy::{EqParams, HdmiPhy, LtpPattern};
use hdmi_hal::scdc::ScdcTransport;

// --- Simulated backends ----------------------------------------------------------

struct SimulatedScdc {
    registers: [u8; 256],
}

impl SimulatedScdc {
    fn new() -> Self {
        Self {
            registers: [0u8; 256],
        }
    }
}

impl ScdcTransport for SimulatedScdc {
    type Error = Infallible;

    fn read(&self, reg: u8) -> Result<u8, Infallible> {
        // `reg` is u8 (0–255) and `registers` has exactly 256 entries, so this
        // index is always in bounds — no panic is possible.
        Ok(self.registers[reg as usize])
    }

    fn write(&mut self, reg: u8, value: u8) -> Result<(), Infallible> {
        // Same guarantee as `read`: u8 address space matches array length exactly.
        self.registers[reg as usize] = value;
        Ok(())
    }
}

struct SimulatedPhy;

impl HdmiPhy for SimulatedPhy {
    type Error = Infallible;

    fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Infallible> {
        println!("PHY: set_frl_rate({rate:?})");
        Ok(())
    }

    fn send_ltp(&mut self, pattern: LtpPattern) -> Result<(), Infallible> {
        println!("PHY: send_ltp({})", pattern.value());
        Ok(())
    }

    fn adjust_equalization(&mut self, _params: EqParams) -> Result<(), Infallible> {
        println!("PHY: adjust_equalization(..)");
        Ok(())
    }

    fn set_scrambling(&mut self, enabled: bool) -> Result<(), Infallible> {
        println!("PHY: set_scrambling({enabled})");
        Ok(())
    }
}

// --- Consumer functions ----------------------------------------------------------
//
// These are the signature patterns that downstream crates (scdc, frl-training,
// platform backends) will follow: generic over the trait, not the concrete type.

fn exercise_scdc<T>(transport: &mut T)
where
    T: ScdcTransport,
    T::Error: core::fmt::Debug,
{
    transport.write(0x20, 0x01).unwrap();
    let val = transport.read(0x20).unwrap();
    println!("SCDC: wrote 0x01 to reg 0x20, read back 0x{val:02x}");
}

fn exercise_phy<P>(phy: &mut P)
where
    P: HdmiPhy,
    P::Error: core::fmt::Debug,
{
    phy.set_frl_rate(HdmiForumFrl::Rate6Gbps4Lanes).unwrap();
    phy.send_ltp(LtpPattern::new(1)).unwrap(); // LFSR0
    phy.adjust_equalization(EqParams::new()).unwrap();
    phy.set_scrambling(true).unwrap();
}

// --------------------------------------------------------------------------------

fn main() {
    let mut scdc = SimulatedScdc::new();
    exercise_scdc(&mut scdc);

    let mut phy = SimulatedPhy;
    exercise_phy(&mut phy);
}
