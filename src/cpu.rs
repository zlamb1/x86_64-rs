#[cfg(not(feature = "std"))]
#[inline]
/// Stops execution until the next interrupt.
/// Note: Caution should be taken not to call this with
/// interrupts disabled as that can deadlock the CPU.
pub fn idle() {
    unsafe {
        core::arch::asm!("hlt", options(nostack, preserves_flags));
    }
}

#[cfg(not(feature = "std"))]
/// Stops execution on the current CPU. Note: NMI and other
/// sources can still wake it.
pub fn stop() -> ! {
    crate::int::disable();
    loop {
        idle();
    }
}
