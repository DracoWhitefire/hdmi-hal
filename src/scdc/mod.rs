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

    /// Read consecutive SCDC registers, starting at `reg`, into `buf`.
    ///
    /// The default implementation calls [`read`](Self::read) once per byte, stopping at
    /// the first error. Transports that can read several bytes in one DDC/I²C
    /// transaction should override it, so that related registers (such as the status
    /// flags or the character error counters) are read together rather than one at a
    /// time while the sink may be updating them. A read past register `0xFF` is not
    /// meaningful; the default implementation wraps the address to `0x00`.
    fn read_block(&self, reg: u8, buf: &mut [u8]) -> Result<(), Self::Error> {
        let mut addr = reg;
        for byte in buf {
            *byte = self.read(addr)?;
            addr = addr.wrapping_add(1);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    struct MockTransport {
        registers: [u8; 256],
        reads: Cell<u32>,
        fail_at: Option<u8>,
    }

    impl MockTransport {
        fn new() -> Self {
            let mut registers = [0u8; 256];
            for (i, r) in registers.iter_mut().enumerate() {
                *r = i as u8;
            }
            Self {
                registers,
                reads: Cell::new(0),
                fail_at: None,
            }
        }
    }

    impl ScdcTransport for MockTransport {
        type Error = u8;

        fn read(&self, reg: u8) -> Result<u8, Self::Error> {
            if self.fail_at == Some(reg) {
                return Err(reg);
            }
            self.reads.set(self.reads.get() + 1);
            Ok(self.registers[reg as usize])
        }

        fn write(&mut self, reg: u8, value: u8) -> Result<(), Self::Error> {
            self.registers[reg as usize] = value;
            Ok(())
        }
    }

    #[test]
    fn read_block_reads_consecutive_registers() {
        let t = MockTransport::new();
        let mut buf = [0u8; 3];
        t.read_block(0x40, &mut buf).unwrap();
        assert_eq!(buf, [0x40, 0x41, 0x42]);
        assert_eq!(t.reads.get(), 3);
    }

    #[test]
    fn read_block_empty_buffer_reads_nothing() {
        let t = MockTransport::new();
        t.read_block(0x40, &mut []).unwrap();
        assert_eq!(t.reads.get(), 0);
    }

    #[test]
    fn read_block_stops_at_first_error() {
        let mut t = MockTransport::new();
        t.fail_at = Some(0x41);
        let mut buf = [0u8; 3];
        assert_eq!(t.read_block(0x40, &mut buf), Err(0x41));
        assert_eq!(buf, [0x40, 0x00, 0x00]);
        assert_eq!(t.reads.get(), 1);
    }

    #[test]
    fn read_block_wraps_past_0xff() {
        let t = MockTransport::new();
        let mut buf = [0u8; 2];
        t.read_block(0xFF, &mut buf).unwrap();
        assert_eq!(buf, [0xFF, 0x00]);
    }

    #[test]
    fn write_then_read_block() {
        let mut t = MockTransport::new();
        t.write(0x20, 0xAB).unwrap();
        let mut buf = [0u8; 1];
        t.read_block(0x20, &mut buf).unwrap();
        assert_eq!(buf, [0xAB]);
    }
}
