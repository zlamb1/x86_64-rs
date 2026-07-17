pub mod desc;

#[repr(C, packed)]
pub struct Gdtr {
    size: u16,
    offset: u64,
}

impl Gdtr {
    /// Constructs a Gdtr from an existing global descriptor table.
    pub fn new<const N: usize>(gdt: &Gdt<N>) -> Self {
        let size = (N * 8) - 1;
        let offset = (&raw const *gdt).addr();

        Self {
            size: size as u16,
            offset: offset as u64,
        }
    }

    /// Loads the GDTR for the current CPU.
    ///
    /// # Safety
    ///
    /// The size and offset must be valid for the global descriptor table.
    /// The global descriptor table must live as long as it is loaded.
    ///
    pub unsafe fn load(&self) {
        unsafe {
            core::arch::asm!("lgdt [{}]", in(reg) &raw const *self, options(nostack, preserves_flags));
        }
    }

    /// Stores the active global descriptor table into this structure.
    ///  
    /// # Safety
    ///
    /// The caller must have sufficient permission to invoke sgdt. Otherwise
    /// a #GP exception can occur when UMIP is enabled and CPL > 0.
    ///
    pub unsafe fn store(&mut self) {
        unsafe {
            core::arch::asm!("sgdt [{}]", in(reg) &raw mut *self, options(nostack, preserves_flags));
        }
    }
}

#[repr(C)]
pub struct Gdt<const N: usize> {
    entries: [u64; N],
    next: usize,
}

impl<const N: usize> Gdt<N> {
    /// Constructs a zeroed global descriptor table.
    pub const fn new() -> Self {
        const {
            assert!(N >= 1);
            assert!(N <= 8192);
        }

        Self {
            entries: [0; _],
            next: 1,
        }
    }

    /// Returns the total capacity of the global descriptor table.
    /// This includes the reserved null entry.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns the total entries in the global descriptor table.
    /// This includes the reserved null entry.
    pub const fn len(&self) -> usize {
        self.next
    }

    /// Returns the raw entries, excluding the first null entry.
    pub fn as_slice(&self) -> &[u64] {
        &self.entries[1..]
    }

    /// Returns the raw mutable entries, excluding the first null entry.
    pub fn as_slice_mut(&mut self) -> &mut [u64] {
        &mut self.entries[1..]
    }

    /// Appends a descriptor to the table.
    pub const fn append(&mut self, desc: desc::Descriptor) {
        match desc.bits() {
            desc::Either::Left(bits) => {
                self.entries[self.next] = bits;
                self.next += 1;
            }
            desc::Either::Right(bits) => {
                self.entries[self.next] = bits as u64;
                self.entries[self.next + 1] = (bits >> 64) as u64;
                self.next += 2;
            }
        }
    }

    /// Loads this global descriptor table.
    pub fn load(&'static self) {
        unsafe {
            self.load_unsafe();
        }
    }

    /// Loads this global descriptor table.
    ///
    /// # Safety
    ///
    /// See [`Gdtr::load`].
    ///
    pub unsafe fn load_unsafe(&self) {
        unsafe {
            Gdtr::new(self).load();
        }
    }
}
