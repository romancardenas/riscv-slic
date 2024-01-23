pub use riscv::interrupt::{disable, enable};

#[cfg(feature = "msoft")]
use riscv::register::mie::{clear_msoft as disable_swi, set_msoft as enable_swi};
#[cfg(feature = "ssoft")]
use riscv::register::sie::{clear_ssoft as disable_swi, set_ssoft as enable_swi};

extern "Rust" {
    fn __riscv_slic_swi_unpend();
    fn __riscv_slic_set_threshold(priority: u8);
    fn __riscv_slic_get_threshold() -> u8;
    fn __riscv_slic_get_priority(interrupt: u16) -> u8;
    fn __riscv_slic_set_priority(interrupt: u16, priority: u8);
    fn __riscv_slic_pend(interrupt: u16);
}

/// Clears software interrupt flags to avoid interruptions.
/// It also resets the software interrupt controllers.
///
/// # Note
///
/// This function does **NOT** modify the [`riscv::register::mstatus`] register.
/// If you want to disable **ANY** other interrupt source, you must **ALSO** use the [`disable`] function.
#[inline]
pub fn clear_interrupts() {
    // SAFETY: interrupts are disabled before modifying thresholds/priorities
    unsafe {
        disable_swi();
        __riscv_slic_swi_unpend();
        set_threshold(u8::MAX);
    }
}

/// Sets the interrupt flags to allow software interrupts.
/// It also sets the interrupt threshold to 0 (i.e., accept all interrupts).
///
/// # Note
///
/// This function does not modify the [`riscv::register::mstatus`] register.
/// If you want to enable **ANY** other interrupt source, you must **ALSO** use the [`enable`] function.
///
/// # Safety
///
/// This function may break mask-based critical sections.
#[inline]
pub unsafe fn set_interrupts() {
    set_threshold(0);
    enable_swi();
}

/// Stabilized API for changing the threshold of the SLIC.
///
/// # Safety
///
/// Changing the priority threshold may break mask-based critical sections.
#[inline]
pub unsafe fn set_threshold(priority: u8) {
    __riscv_slic_set_threshold(priority);
}

/// Stabilized API for getting the current threshold of the SLIC.
#[inline]
pub fn get_threshold() -> u8 {
    // SAFETY: this read has no side effects.
    unsafe { __riscv_slic_get_threshold() }
}

/// Stabilized API for getting the priority of a given software interrupt source.
#[inline]
pub fn get_priority<I: crate::InterruptNumber>(interrupt: I) -> u8 {
    // SAFETY: this read has no side effects.
    unsafe { __riscv_slic_get_priority(interrupt.number()) }
}

/// Stabilized API for setting the priority of a software interrupt of the SLIC.
///
/// # Safety
///
/// Changing the priority of an interrupt may break mask-based critical sections.
#[inline]
pub unsafe fn set_priority<I: crate::InterruptNumber>(interrupt: I, priority: u8) {
    __riscv_slic_set_priority(interrupt.number(), priority);
}

/// Stabilized API for pending a software interrupt on the SLIC.
#[inline]
pub fn pend<I: crate::InterruptNumber>(interrupt: I) {
    // SAFETY: TODO
    unsafe { __riscv_slic_pend(interrupt.number()) };
}

/// Runs a function with priority mask.
///
/// # Safety
///
/// If new priority is less than current priority, priority inversion may occur.
#[inline]
pub unsafe fn run<F: FnOnce()>(priority: u8, f: F) {
    let current = get_threshold();
    set_threshold(priority);
    f();
    set_threshold(current);
}

/// Runs a function that takes a shared resource with a priority ceiling.
/// This function returns the return value of the target function.
///
/// # Safety
///
/// If ceiling is less than current priority, priority inversion may occur.
#[inline]
pub unsafe fn lock<F, T, R>(ptr: *mut T, ceiling: u8, f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    let current = get_threshold();
    set_threshold(ceiling);
    let r = f(&mut *ptr);
    set_threshold(current);
    r
}
