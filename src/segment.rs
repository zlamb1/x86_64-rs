use core::arch::asm;

use crate::Dpl;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Segment {
    CS,
    DS,
    ES,
    FS,
    GS,
    SS,
}

impl Segment {
    /// Read the current value of the segment register.
    #[inline]
    pub fn read(self) -> Selector {
        let selector: u16;

        unsafe {
            match self {
                Segment::CS => {
                    asm!("mov {:x}, cs", out(reg) selector, options(nomem, nostack, preserves_flags))
                }
                Segment::DS => {
                    asm!("mov {:x}, ds", out(reg) selector, options(nomem, nostack, preserves_flags))
                }
                Segment::ES => {
                    asm!("mov {:x}, es", out(reg) selector, options(nomem, nostack, preserves_flags))
                }
                Segment::FS => {
                    asm!("mov {:x}, fs", out(reg) selector, options(nomem, nostack, preserves_flags))
                }
                Segment::GS => {
                    asm!("mov {:x}, gs", out(reg) selector, options(nomem, nostack, preserves_flags))
                }
                Segment::SS => {
                    asm!("mov {:x}, ss", out(reg) selector, options(nomem, nostack, preserves_flags))
                }
            }
        }

        selector.into()
    }
}

impl TryFrom<Segment> for Writable {
    type Error = ();

    fn try_from(segment: Segment) -> Result<Self, Self::Error> {
        Ok(match segment {
            Segment::CS => return Err(()),
            Segment::DS => Self::DS,
            Segment::ES => Self::ES,
            Segment::FS => Self::FS,
            Segment::GS => Self::GS,
            Segment::SS => Self::SS,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Writable {
    DS,
    ES,
    FS,
    GS,
    SS,
}

impl Writable {
    /// Write a value into the segment register.
    ///
    /// # Safety
    ///
    /// The value written must be a valid selector into the global descriptor table.
    #[inline]
    pub unsafe fn write(self, selector: Selector) {
        let selector = selector.bits();

        unsafe {
            match self {
                Self::DS => {
                    asm!("mov ds, {:x}", in(reg) selector, options(nomem, nostack, preserves_flags))
                }
                Self::ES => {
                    asm!("mov es, {:x}", in(reg) selector, options(nomem, nostack, preserves_flags))
                }
                Self::FS => {
                    asm!("mov fs, {:x}", in(reg) selector, options(nomem, nostack, preserves_flags))
                }
                Self::GS => {
                    asm!("mov gs, {:x}", in(reg) selector, options(nomem, nostack, preserves_flags))
                }
                Self::SS => {
                    asm!("mov ss, {:x}", in(reg) selector, options(nomem, nostack, preserves_flags))
                }
            }
        }
    }

    pub fn as_segment(self) -> Segment {
        match self {
            Writable::DS => Segment::DS,
            Writable::ES => Segment::ES,
            Writable::FS => Segment::FS,
            Writable::GS => Segment::GS,
            Writable::SS => Segment::SS,
        }
    }
}

impl From<Writable> for Segment {
    fn from(writable: Writable) -> Self {
        writable.as_segment()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum TableIndex {
    GDT = 0,
    LDT = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Selector(u16);

impl Selector {
    pub const fn from_index(index: u16) -> Self {
        Self(index << 3)
    }

    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn bits(&self) -> u16 {
        self.0
    }

    /// The requested privilege level of the selector.
    pub const fn rpl(&self) -> Dpl {
        let dpl = self.0 & 0x3;

        match dpl {
            0 => Dpl::Ring0,
            1 => Dpl::Ring1,
            2 => Dpl::Ring2,
            3 => Dpl::Ring3,
            _ => unreachable!(),
        }
    }

    pub const fn with_rpl(mut self, rpl: Dpl) -> Self {
        self.0 &= !0x3;
        self.0 |= rpl as u16;
        self
    }

    pub const fn set_rpl(&mut self, rpl: Dpl) {
        self.0 &= !0x3;
        self.0 |= rpl as u16;
    }

    pub const fn table_index(&self) -> TableIndex {
        let ti = self.0 >> 2 & 0x1;

        match ti {
            0 => TableIndex::GDT,
            1 => TableIndex::LDT,
            _ => unreachable!(),
        }
    }

    pub const fn with_table_index(mut self, table_index: TableIndex) -> Self {
        self.0 &= !(0x1 << 2);
        self.0 |= (table_index as u16) << 2;
        self
    }

    pub const fn set_table_index(&mut self, table_index: TableIndex) {
        self.0 &= !(0x1 << 2);
        self.0 |= (table_index as u16) << 2;
    }

    pub const fn index(&self) -> u16 {
        self.0 >> 3
    }

    pub const fn with_index(mut self, index: u16) -> Self {
        self.0 &= !0x7;
        self.0 |= index << 3;
        self
    }

    pub const fn set_index(&mut self, index: u16) {
        self.0 &= !0x7;
        self.0 |= index << 3;
    }
}

impl From<u16> for Selector {
    fn from(x: u16) -> Self {
        Self(x)
    }
}

impl Default for Selector {
    fn default() -> Self {
        Self::null()
    }
}
