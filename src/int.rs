use core::arch::asm;

#[inline]
pub fn enable() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

#[inline]
pub fn disable() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

#[inline]
pub fn save() -> usize {
    let flags: usize;
    unsafe {
        asm!("pushfq; popq {}", out(reg) flags, options(nomem, preserves_flags));
    }
    flags
}

#[inline]
pub fn save_and_disable() -> usize {
    let flags: usize;
    unsafe {
        asm!("pushfq; popq {}; cli", out(reg) flags, options(nomem));
    }
    flags
}

#[inline]
pub fn restore(flags: usize) {
    unsafe {
        asm!("pushq {}; popfq", in(reg) flags, options(nomem));
    }
}
