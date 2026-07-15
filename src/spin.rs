use core::arch::asm;

/// Hints to the CPU about a spin-wait loop.
pub fn hint() {
    unsafe {
        asm!("pause", options(nomem, nostack, preserves_flags));
    }
}
