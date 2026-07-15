#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PageFault(u32);

impl PageFault {
    const PRESENT: u32 = 0x1;
    const WRITE: u32 = 0x2;
    const USER: u32 = 0x4;
    const RESERVED: u32 = 0x8;
    const INSTRUCTION_FETCH: u32 = 0x10;
    const PROTECTION_KEY: u32 = 0x20;
    const SHADOW_STACK: u32 = 0x40;

    pub const fn new(error_code: u32) -> Self {
        Self(error_code)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn was_page_present(&self) -> bool {
        self.0 & Self::PRESENT != 0
    }

    pub const fn was_write(&self) -> bool {
        self.0 & Self::WRITE != 0
    }

    pub const fn was_user(&self) -> bool {
        self.0 & Self::USER != 0
    }

    /// One or more page table entries contain reserved bits which are set to 1.
    pub const fn was_malformed_page_table(&self) -> bool {
        self.0 & Self::RESERVED != 0
    }

    pub const fn was_instruction_fetch(&self) -> bool {
        self.0 & Self::INSTRUCTION_FETCH != 0
    }

    pub const fn was_protection_key_violation(&self) -> bool {
        self.0 & Self::PROTECTION_KEY != 0
    }

    pub const fn was_shadow_stack_access(&self) -> bool {
        self.0 & Self::SHADOW_STACK != 0
    }
}
