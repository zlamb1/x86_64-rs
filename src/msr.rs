use core::arch::asm;

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Msr {
    Ia32Efer = 0xC0000080,
    FsBase = 0xC0000100,
    GsBase = 0xC0000101,
    KernelGsBase = 0xC0000102,
}

impl Msr {
    /// Reads a value from the model specific register.
    ///
    /// # Safety
    ///
    /// The caller must ensure the model specific register
    /// is supported by the processor. Failure to do so
    /// will result in a #GP exception.
    #[must_use]
    pub unsafe fn read(self) -> u64 {
        let lo: u32;
        let hi: u32;
        let msr = self as u32;

        unsafe {
            asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi, options(nomem, nostack, preserves_flags));
        }

        (lo as u64) | (hi as u64) << 32
    }

    /// Writes a value into the model specific register.
    ///
    /// # Safety
    ///
    /// The caller must ensure the model specific register
    /// is supported by the processor. Failure to do so
    /// will result in a #GP exception. The caller must also
    /// ensure that any invariants of the model specific register's
    /// value are maintained.
    pub unsafe fn write(self, value: u64) {
        let lo: u32 = value as u32;
        let hi: u32 = (value >> 32) as u32;
        let msr = self as u32;

        unsafe {
            asm!("wrmsr", in("ecx") msr, in("eax") lo, in("edx") hi, options(nomem, nostack, preserves_flags))
        }
    }

    /// Provides batched read-modify-writes over the model specific register.
    ///
    /// # Safety
    /// See [`Msr::write`].
    pub unsafe fn rmw(self, f: impl FnOnce(&mut Rmw)) {
        let mut rmw = Rmw::new(self);
        f(&mut rmw);
        rmw.commit();
    }
}

pub struct Rmw {
    msr: Msr,
    original: u64,
    current: u64,
    new: u64,
}

impl Rmw {
    fn new(msr: Msr) -> Self {
        let original = unsafe { msr.read() };
        Self {
            msr,
            original,
            current: original,
            new: original,
        }
    }

    fn mask(bit: usize, len: usize) -> u64 {
        assert!(bit < 64);

        let end = bit.checked_add(len).unwrap();

        assert!(end <= 64);

        if len < 64 {
            (1u64 << len) - 1
        } else {
            u64::MAX
        }
    }

    /// Write the new value to the model specific register.
    pub fn commit(&mut self) {
        if self.current != self.new {
            self.current = self.new;

            unsafe {
                self.msr.write(self.new);
            }
        }
    }

    /// Discards the new value and restores the old value to the model specific register if necessary.
    pub fn rollback(&mut self) {
        self.new = self.original;

        if self.original != self.current {
            self.current = self.original;

            unsafe {
                self.msr.write(self.original);
            }
        }
    }

    /// Reads the new value.
    #[must_use]
    pub fn read(&self) -> u64 {
        self.new
    }

    #[must_use]
    pub fn read_bit(&self, bit: usize) -> bool {
        assert!(bit < 64);

        (self.new >> bit) & 1 == 1
    }

    #[must_use]
    pub fn read_bits(&self, bit: usize, len: usize) -> u64 {
        let mask = Self::mask(bit, len);

        self.new >> bit & mask
    }

    /// Zeroes the new value.
    pub fn zero(&mut self) {
        self.new = 0;
    }

    /// Writes the new value.
    pub fn write(&mut self, new: u64) {
        self.new = new;
    }

    pub fn write_bit(&mut self, bit: usize, set: bool) {
        assert!(bit < 64);

        if set {
            self.new |= 1u64 << bit;
        } else {
            self.new &= !(1u64 << bit);
        }
    }

    pub fn write_bits(&mut self, bit: usize, len: usize, value: u64) {
        let mask = Self::mask(bit, len);

        self.new &= !(mask << bit);
        self.new |= (value & mask) << bit;
    }
}
