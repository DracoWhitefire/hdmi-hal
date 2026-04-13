/// Raw register access to the SCDC register map over DDC/I²C.
///
/// SCDC is a bidirectional channel used for FRL link training initiation, scrambling
/// control, and CED (Character Error Detection) reporting. Any code that needs to touch
/// the SCDC register map does so through this trait.
///
/// The trait operates at the raw register level: a one-byte address and a one-byte
/// value. Typed register wrappers, named constants, and multi-register sequences belong
/// in the `scdc` crate, not here.
pub trait ScdcTransport {
    /// Error type returned by transport operations.
    type Error;

    /// Read a single byte from the given SCDC register address.
    fn read(&self, reg: u8) -> Result<u8, Self::Error>;

    /// Write a single byte to the given SCDC register address.
    fn write(&mut self, reg: u8, value: u8) -> Result<(), Self::Error>;
}
