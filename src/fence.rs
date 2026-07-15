use core::arch::asm;

#[inline]
pub fn compiler() {
    unsafe {
        asm!("", options(nostack, preserves_flags));
    }
}

#[inline]
pub fn load() {
    // Note: Nothing further necessary on x86_64. Empty asm! is to inhibit
    // compiler reordering.
    compiler();
}

#[inline]
pub fn store() {
    unsafe {
        asm!("sfence", options(nostack, preserves_flags));
    }
}

#[inline]
pub fn memory() {
    unsafe {
        asm!("mfence", options(nostack, preserves_flags));
    }
}
