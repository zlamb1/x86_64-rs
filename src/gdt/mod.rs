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
    len: usize,
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
            len: 1,
        }
    }

    /// Returns the total capacity of the global descriptor table, including the reserved null entry.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns the total entries in the global descriptor table, including the reserved null entry.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns the raw entries, excluding the reserved null entry.
    ///
    /// #### Note: This also yields null (zeroed) entries beyond [`Gdt::len`].
    ///
    pub fn as_slice(&self) -> &[u64] {
        &self.entries[1..]
    }

    /// Returns the raw mutable entries, excluding the reserved null entry.
    ///
    /// #### Note: This also yields null (zeroed) entries beyond [`Gdt::len`].
    ///
    pub fn as_mut_slice(&mut self) -> &mut [u64] {
        &mut self.entries[1..]
    }

    /// Returns the raw mutable entries. including the reserved null entry.
    ///
    /// #### Note: This also yields null (zeroed) entries beyond [`Gdt::len`].
    ///
    /// # Safety
    ///
    /// The caller must take care to preserve the reserved null entry.
    /// If the global descriptor table is loaded with a non-null first entry,
    /// it can result in a #GP exception.
    ///
    pub unsafe fn as_mut_slice_unchecked(&mut self) -> &mut [u64] {
        &mut self.entries
    }

    /// Appends a descriptor to the table.
    pub const fn append(&mut self, desc: desc::Descriptor) {
        match desc.bits() {
            desc::Either::Left(bits) => {
                self.entries[self.len] = bits;
                self.len += 1;
            }
            desc::Either::Right(bits) => {
                self.entries[self.len] = bits as u64;
                self.entries[self.len + 1] = (bits >> 64) as u64;
                self.len += 2;
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

    /// Returns an iterator over the entries, excluding the reserved null entry.
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }
}

pub struct Iter<'a> {
    entries: &'a [u64],
    index: usize,
}

impl<'a> Iter<'a> {
    pub fn new<const N: usize>(gdt: &'a Gdt<N>) -> Self {
        Self {
            entries: &gdt.entries[..gdt.len()],
            index: 1,
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = desc::Result<desc::Descriptor>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entries.len() {
            let entries = &self.entries[self.index..];
            let desc: desc::Result<desc::Descriptor> = entries.try_into();
            match &desc {
                Ok(desc) => self.index += desc.entries(),
                Err(_) => self.index += 1,
            }
            return Some(desc);
        }
        None
    }
}
