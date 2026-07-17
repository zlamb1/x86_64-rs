use core::ops::{Index, IndexMut};

use crate::{
    Dpl,
    segment::{self, Selector},
};

#[cfg(not(feature = "std"))]
#[repr(C, packed)]
struct Idtr {
    size: u16,
    offset: u64,
}

#[cfg(not(feature = "std"))]
impl Idtr {
    fn from_idt<const N: usize>(idt: &Idt<N>) -> Self {
        Self {
            size: (N * 16 - 1) as u16,
            offset: (&raw const *idt).addr() as u64,
        }
    }

    unsafe fn load(&self) {
        let addr = (&raw const *self).addr();

        unsafe {
            core::arch::asm!("lidt [{}]", in(reg) addr, options(nostack, preserves_flags));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Idt<const N: usize> {
    descriptors: [Descriptor; N],
}

impl<const N: usize> Idt<N> {
    pub const fn new() -> Self {
        const {
            assert!(N > 0);
            assert!(N <= 256);
        }

        Self {
            descriptors: [Descriptor::null(); _],
        }
    }

    pub const fn len() -> usize {
        N
    }

    #[cfg(not(feature = "std"))]
    pub fn load(&'static self) {
        unsafe {
            self.load_unsafe();
        }
    }

    /// # Safety
    ///
    /// The IDT must remain valid as long as its in use.
    #[cfg(not(feature = "std"))]
    pub unsafe fn load_unsafe(&self) {
        let idtr = Idtr::from_idt(self);
        unsafe {
            idtr.load();
        }
    }
}

impl<const N: usize> Default for Idt<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Index<usize> for Idt<N> {
    type Output = Descriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.descriptors[index]
    }
}

impl<const N: usize> IndexMut<usize> for Idt<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.descriptors[index]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Gate {
    Interrupt = 0xE,
    Trap = 0xF,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Descriptor {
    offset_lo: u16,
    selector: segment::Selector,
    options: u16,
    offset_mid: u16,
    offset_hi: u32,
    reserved: u32,
}

impl Descriptor {
    /// Constructs a new IDT descriptor with a given offset. The new
    /// descriptor is marked present, is an interrupt gate, and uses the active code segment.
    pub fn new(offset: u64) -> Self {
        Self::null()
            .with_offset(offset)
            .with_segment_selector(segment::Segment::CS.read())
            .with_gate_type(Gate::Interrupt)
            .with_present(true)
    }

    /// Constructs a new IDT descriptor with a given offset and code segment. The new
    /// descriptor is marked present and is an interrupt gate.
    pub const fn new_with(offset: u64, cs: Selector) -> Self {
        Self::null()
            .with_offset(offset)
            .with_segment_selector(cs)
            .with_gate_type(Gate::Interrupt)
            .with_present(true)
    }

    pub const fn null() -> Self {
        Self {
            offset_lo: 0,
            selector: Selector::null(),
            options: 0,
            offset_mid: 0,
            offset_hi: 0,
            reserved: 0,
        }
    }

    pub const fn offset(&self) -> u64 {
        self.offset_lo as u64 | (self.offset_mid as u64) << 16 | (self.offset_hi as u64) << 32
    }

    pub const fn with_offset(mut self, offset: u64) -> Self {
        self.set_offset(offset);
        self
    }

    pub const fn set_offset(&mut self, offset: u64) {
        self.offset_lo = offset as u16;
        self.offset_mid = (offset >> 16) as u16;
        self.offset_hi = (offset >> 32) as u32;
    }

    pub const fn segment_selector(&self) -> Selector {
        self.selector
    }

    pub const fn with_segment_selector(mut self, selector: Selector) -> Self {
        self.selector = selector;
        self
    }

    pub const fn set_segment_selector(&mut self, selector: Selector) {
        self.selector = selector;
    }

    pub const fn gate_type(&self) -> Gate {
        let gate_type = (self.options >> 8) & 0xF;

        match gate_type {
            0xE => Gate::Interrupt,
            0xF => Gate::Trap,
            _ => unreachable!(),
        }
    }

    pub const fn with_gate_type(mut self, gate_type: Gate) -> Self {
        self.set_gate_type(gate_type);
        self
    }

    pub const fn set_gate_type(&mut self, gate_type: Gate) {
        self.options &= !(0xF << 8);
        self.options |= (gate_type as u16) << 8;
    }

    pub const fn dpl(&self) -> Dpl {
        let dpl = (self.options >> 13) & 0x3;

        match dpl {
            0 => Dpl::Ring0,
            1 => Dpl::Ring1,
            2 => Dpl::Ring2,
            3 => Dpl::Ring3,
            _ => unreachable!(),
        }
    }

    pub const fn with_dpl(mut self, dpl: Dpl) -> Self {
        self.set_dpl(dpl);
        self
    }

    pub const fn set_dpl(&mut self, dpl: Dpl) {
        self.options &= !(0x3 << 13);
        self.options |= (dpl as u16) << 13;
    }

    pub const fn present(&self) -> bool {
        self.options >> 15 & 0x1 == 0x1
    }

    pub const fn with_present(mut self, present: bool) -> Self {
        self.set_present(present);
        self
    }

    pub const fn set_present(&mut self, present: bool) {
        if present {
            self.options |= 0x1 << 15;
        } else {
            self.options &= !(0x1 << 15);
        }
    }
}
