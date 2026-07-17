use crate::Dpl;

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub enum Descriptor {
    Null,
    Memory(Memory),
}

impl Descriptor {
    /// Returns the raw bits of the underlying representation.
    pub const fn bits(&self) -> Either<u64, u128> {
        match self {
            Descriptor::Null => Either::Left(0),
            Descriptor::Memory(memory) => Either::Left(memory.bits()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Protected mode (16-bit).
    Prot16,
    /// Protected mode (32-bit).
    Prot32,
    /// Long mode (64-bit). Valid only for long-mode code segments.
    Long,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Granularity {
    Byte,
    Page,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Memory {
    limit_0_15: u16,
    base_0_15: u16,
    base_16_23: u8,
    access_0_7: u8,
    limit_16_19_flags_0_3: u8,
    base_24_31: u8,
}

impl Memory {
    const ACCESSED_0_1: u8 = 0x1;
    const READABLE_0_1: u8 = 0x2;
    const CONFORMING_0_1: u8 = 0x4;
    const EXECUTABLE_0_1: u8 = 0x8;
    const SYSTEM_0_1: u8 = 0x10;
    const DPL_0_2: u8 = 0x60;
    const DPL_0_2_BIT: u8 = 5;
    const PRESENT_0_1: u8 = 0x80;
    const LIMIT_16_19: u8 = 0xF;
    const LONG_0_1: u8 = 0x20;
    const DB_0_1: u8 = 0x40;
    const GRANULARITY_0_1: u8 = 0x80;

    /// Constructs a zeroed memory segment, except for bits that must be set.
    pub const fn new() -> Self {
        Self {
            limit_0_15: 0,
            base_0_15: 0,
            base_16_23: 0,
            access_0_7: Self::SYSTEM_0_1,
            limit_16_19_flags_0_3: 0,
            base_24_31: 0,
        }
    }

    /// Creates a code segment suitable for use in a long-mode kernel.
    pub const fn kernel_code_segment() -> Self {
        Self::new()
            .with_max_limit()
            .with_code_segment()
            .with_readable(true)
            .with_conforming(false)
            .with_dpl(Dpl::Ring0)
            .with_present(true)
            .with_mode(Mode::Long)
            .with_granularity(Granularity::Page)
    }

    /// Creates a data segment suitable for use in a long-mode kernel.
    pub const fn kernel_data_segment() -> Self {
        Self::new()
            .with_max_limit()
            .with_data_segment()
            .with_writable(true)
            .with_direction(Direction::Up)
            .with_dpl(Dpl::Ring0)
            .with_present(true)
            .with_mode(Mode::Prot32)
            .with_granularity(Granularity::Page)
    }

    /// Creates a code segment suitable for use in long-mode user space.
    pub const fn user_code_segment() -> Self {
        Self::new()
            .with_max_limit()
            .with_code_segment()
            .with_readable(true)
            .with_conforming(false)
            .with_dpl(Dpl::Ring3)
            .with_present(true)
            .with_mode(Mode::Long)
            .with_granularity(Granularity::Page)
    }

    /// Creates a data segment suitable for use in long-mode user space.
    pub const fn user_data_segment() -> Self {
        Self::new()
            .with_max_limit()
            .with_data_segment()
            .with_writable(true)
            .with_direction(Direction::Up)
            .with_dpl(Dpl::Ring3)
            .with_present(true)
            .with_mode(Mode::Prot32)
            .with_granularity(Granularity::Page)
    }

    /// Returns the raw bits of this descriptor.
    pub const fn bits(&self) -> u64 {
        self.limit_0_15 as u64
            | (self.base_0_15 as u64) << 16
            | (self.base_16_23 as u64) << 32
            | (self.access_0_7 as u64) << 40
            | (self.limit_16_19_flags_0_3 as u64) << 48
            | (self.base_24_31 as u64) << 56
    }

    /// Converts the descriptor into raw bits.
    pub const fn into_bits(self) -> u64 {
        self.bits()
    }

    /// Returns the limit of the memory segment.
    pub const fn limit(&self) -> u32 {
        self.limit_0_15 as u32
            | (self.limit_16_19_flags_0_3 as u32 & Self::LIMIT_16_19 as u32) << 16
    }

    /// Sets the limit of the memory segment.
    ///
    /// # Panics
    ///
    /// If `limit` exceeds 0xFFFFF.
    ///
    pub const fn set_limit(&mut self, limit: u32) {
        assert!(limit <= 0xFFFFF);
        self.limit_0_15 = limit as u16;
        self.limit_16_19_flags_0_3 &= !Self::LIMIT_16_19;
        self.limit_16_19_flags_0_3 |= (limit >> 16) as u8;
    }

    /// Builds a new memory segment with `limit`.
    ///
    /// # Panics
    ///
    /// If `limit` exceeds 0xFFFFF.
    ///
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.set_limit(limit);
        self
    }

    /// Builds a new memory segment with the maximum limit.
    pub const fn with_max_limit(self) -> Self {
        self.with_limit(0xFFFFF)
    }

    /// Returns the base of the memory segment.
    pub const fn base(&self) -> u32 {
        self.base_0_15 as u32 | (self.base_16_23 as u32) << 16 | (self.base_24_31 as u32) << 24
    }

    /// Sets the base of the memory segment.
    pub const fn set_base(&mut self, base: u32) {
        self.base_0_15 = base as u16;
        self.base_16_23 = (base >> 16) as u8;
        self.base_24_31 = (base >> 24) as u8;
    }

    /// Builds a memory segment with `base`.
    pub const fn with_base(mut self, base: u32) -> Self {
        self.set_base(base);
        self
    }

    /// Returns whether the CPU has accessed the descriptor.
    pub const fn was_accessed(&self) -> bool {
        self.access_0_7 & Self::ACCESSED_0_1 != 0
    }

    /// Sets whether the descriptor has been accessed. Note that
    /// this bypasses the CPU which usually sets the relevant bit.
    pub const fn set_accessed(&mut self, accessed: bool) {
        if accessed {
            self.access_0_7 |= Self::ACCESSED_0_1;
        } else {
            self.access_0_7 &= !Self::ACCESSED_0_1;
        }
    }

    /// Builds a memory segment with `accessed`.
    pub const fn with_accessed(mut self, accessed: bool) -> Self {
        self.set_accessed(accessed);
        self
    }

    /// Returns whether this code segment is readable. Aliases with writable for data segments.
    pub const fn readable(&self) -> bool {
        self.access_0_7 & Self::READABLE_0_1 != 0
    }

    /// Sets whether the code segment is readable or not.
    pub const fn set_readable(&mut self, readable: bool) {
        if readable {
            self.access_0_7 |= Self::READABLE_0_1;
        } else {
            self.access_0_7 &= !Self::READABLE_0_1;
        }
    }

    /// Builds a memory segment with `readable`.
    pub const fn with_readable(mut self, readable: bool) -> Self {
        self.set_readable(readable);
        self
    }

    /// Returns whether this data segment is writable. Aliases with readable for code segments.
    pub const fn writable(&self) -> bool {
        self.readable()
    }

    /// Sets whether the data segment is readable or not.
    pub const fn set_writable(&mut self, writable: bool) {
        self.set_readable(writable);
    }

    /// Builds a memory segment with `writable`.
    pub const fn with_writable(mut self, writable: bool) -> Self {
        self.set_writable(writable);
        self
    }

    /// Returns whether the code segment is conforming. If conforming, this segment
    /// can be executed from an equal or lower privilege level. Aliases with `direction` for data segments.
    pub const fn conforming(&self) -> bool {
        self.access_0_7 & Self::CONFORMING_0_1 != 0
    }

    /// Sets whether the code segment is conforming or not.
    pub const fn set_conforming(&mut self, conforming: bool) {
        if conforming {
            self.access_0_7 |= Self::CONFORMING_0_1;
        } else {
            self.access_0_7 &= !Self::CONFORMING_0_1;
        }
    }

    /// Builds a memory segment with `conforming`.
    pub const fn with_conforming(mut self, conforming: bool) -> Self {
        self.set_conforming(conforming);
        self
    }

    /// Returns the growth direction for the data segment. Aliases with conforming for code segments.
    pub const fn direction(&self) -> Direction {
        match self.conforming() {
            false => Direction::Up,
            true => Direction::Down,
        }
    }

    /// Sets whether the data segment grows up or down.
    pub const fn set_direction(&mut self, direction: Direction) {
        self.set_conforming(match direction {
            Direction::Up => false,
            Direction::Down => true,
        });
    }

    /// Builds a memory segment with `direction`.
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.set_direction(direction);
        self
    }

    /// Returns whether this memory segment is a code segment.
    pub const fn is_code_segment(&self) -> bool {
        self.access_0_7 & Self::EXECUTABLE_0_1 != 0
    }

    /// Converts the memory segment into a code segment.
    pub const fn set_code_segment(&mut self) {
        self.access_0_7 |= Self::EXECUTABLE_0_1;
    }

    /// Builds a code segment.
    pub const fn with_code_segment(mut self) -> Self {
        self.set_code_segment();
        self
    }

    /// Returns whether this memory segment is a data segment.
    pub const fn is_data_segment(&self) -> bool {
        self.access_0_7 & Self::EXECUTABLE_0_1 == 0
    }

    /// Converts the memory segment into a data segment.
    ///
    /// # Panics
    ///
    /// Panics if the memory segment's mode is [`Mode::Long`].
    ///
    pub const fn set_data_segment(&mut self) {
        match self.mode() {
            Mode::Long => {
                panic!();
            }
            _ => {}
        }

        self.access_0_7 &= !Self::EXECUTABLE_0_1;
    }

    /// Builds a data segment.
    pub const fn with_data_segment(mut self) -> Self {
        self.set_data_segment();
        self
    }

    /// Returns the privilege level of the memory segment.
    pub const fn dpl(&self) -> Dpl {
        let dpl = (self.access_0_7 & Self::DPL_0_2) >> Self::DPL_0_2_BIT;

        match dpl {
            0 => Dpl::Ring0,
            1 => Dpl::Ring1,
            2 => Dpl::Ring2,
            3 => Dpl::Ring3,
            _ => unreachable!(),
        }
    }

    /// Set the privilege level for this memory segment.
    pub const fn set_dpl(&mut self, dpl: Dpl) {
        let dpl = dpl as u8;
        self.access_0_7 &= !Self::DPL_0_2;
        self.access_0_7 |= dpl << Self::DPL_0_2_BIT;
    }

    /// Builds a memory segment with `dpl`.
    pub const fn with_dpl(mut self, dpl: Dpl) -> Self {
        self.set_dpl(dpl);
        self
    }

    /// Returns whether this descriptor is present. If not set,
    /// the descriptor is considered invalid by the CPU.
    pub const fn present(&self) -> bool {
        self.access_0_7 & Self::PRESENT_0_1 != 0
    }

    /// Sets whether the descriptor is present.
    pub const fn set_present(&mut self, present: bool) {
        if present {
            self.access_0_7 |= Self::PRESENT_0_1;
        } else {
            self.access_0_7 &= !Self::PRESENT_0_1;
        }
    }

    /// Builds a memory segment with `present`.
    pub const fn with_present(mut self, present: bool) -> Self {
        self.set_present(present);
        self
    }

    /// Returns the operating mode of this memory segment.
    pub const fn mode(&self) -> Mode {
        if self.limit_16_19_flags_0_3 & Self::LONG_0_1 != 0 {
            Mode::Long
        } else if self.limit_16_19_flags_0_3 & Self::DB_0_1 != 0 {
            Mode::Prot32
        } else {
            Mode::Prot16
        }
    }

    /// Sets the operating mode of the memory segment.
    ///
    /// # Panics
    ///
    /// Panics if mode is [`Mode::Long`] and this memory segment is not a code segment.
    ///
    pub const fn set_mode(&mut self, mode: Mode) {
        match mode {
            Mode::Prot16 => {
                self.limit_16_19_flags_0_3 &= !(Self::DB_0_1 | Self::LONG_0_1);
            }
            Mode::Prot32 => {
                self.limit_16_19_flags_0_3 &= !Self::LONG_0_1;
                self.limit_16_19_flags_0_3 |= Self::DB_0_1;
            }
            Mode::Long => {
                assert!(self.is_code_segment());
                self.limit_16_19_flags_0_3 &= !Self::DB_0_1;
                self.limit_16_19_flags_0_3 |= Self::LONG_0_1;
            }
        }
    }

    /// Sets the operating mode of the memory segment.
    ///
    /// # Panics
    ///
    /// Panics if mode is [`Mode::Long`] and this memory segment is not a code segment.
    ///
    pub const fn with_mode(mut self, mode: Mode) -> Self {
        self.set_mode(mode);
        self
    }

    /// Returns the granularity of the memory segment. The granularity scales
    /// the limit. For [`Granularity::Byte`], limit is scaled by 1. And for
    /// [`Granularity::Page`], limit is scaled by 4096.
    pub const fn granularity(&self) -> Granularity {
        if self.limit_16_19_flags_0_3 & Self::GRANULARITY_0_1 != 0 {
            Granularity::Page
        } else {
            Granularity::Byte
        }
    }

    /// Sets the granularity of the memory segment.
    pub const fn set_granularity(&mut self, granularity: Granularity) {
        match granularity {
            Granularity::Byte => self.limit_16_19_flags_0_3 &= !Self::GRANULARITY_0_1,
            Granularity::Page => self.limit_16_19_flags_0_3 |= Self::GRANULARITY_0_1,
        }
    }

    /// Builds a memory segment with `granularity`.
    pub const fn with_granularity(mut self, granularity: Granularity) -> Self {
        self.set_granularity(granularity);
        self
    }
}
