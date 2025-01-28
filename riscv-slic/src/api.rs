pub use riscv::interrupt::nested;

use riscv::interrupt::{disable as disable_interrupts, enable as enable_interrupts};
#[cfg(feature = "msoft")]
use riscv::register::mie::{clear_msoft as disable_swi, set_msoft as enable_swi};
#[cfg(feature = "ssoft")]
use riscv::register::sie::{clear_ssoft as disable_swi, set_ssoft as enable_swi};

extern "Rust" {
    #[cfg(any(feature = "msoft", feature = "ssoft"))]
    fn __riscv_slic_swi_unpend();
    fn __riscv_slic_enable();
    fn __riscv_slic_disable();
    fn __riscv_slic_get_threshold() -> u8;
    fn __riscv_slic_set_threshold(priority: u8);
    fn __riscv_slic_raise_threshold(priority: u8) -> Result<u8, ()>;
    fn __riscv_slic_get_priority(interrupt: u16) -> u8;
    fn __riscv_slic_set_priority(interrupt: u16, priority: u8);
    fn __riscv_slic_pend(interrupt: u16);
}

/// Enables the SLIC, software interrupts (if needed), and system interrupts.
///
/// # Safety
///
/// This function may break mask-based critical sections.
#[inline]
pub unsafe fn enable() {
    __riscv_slic_enable();
    #[cfg(any(feature = "msoft", feature = "ssoft"))]
    enable_swi();
    enable_interrupts();
}

/// Disables system interrupts, software interrupts (if needed), and the SLIC.
#[inline]
pub fn disable() {
    disable_interrupts();
    #[cfg(any(feature = "msoft", feature = "ssoft"))]
    // SAFETY: it is safe to clear software interrupt flags
    unsafe {
        disable_swi();
        // __riscv_slic_swi_unpend();
    }
    // SAFETY: interrupts are disabled before disabling SLIC
    unsafe { __riscv_slic_disable() };
}

/// Stabilized API for getting the current threshold of the SLIC.
#[inline]
pub fn get_threshold() -> u8 {
    // SAFETY: this read has no side effects.
    unsafe { __riscv_slic_get_threshold() }
}

/// Stabilized API for setting the threshold of the SLIC.
///
/// # Safety
///
/// Setting the priority threshold to a value lower than the current may lead to priority inversion.
#[inline]
pub unsafe fn set_threshold(priority: u8) {
    __riscv_slic_set_threshold(priority);
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
    // SAFETY: it is safe to pend a software interrupt
    unsafe { __riscv_slic_pend(interrupt.number()) };
}

/// Runs a function with priority mask.
#[inline]
pub fn run<F: FnOnce()>(priority: u8, f: F) {
    // SAFETY: we restore the previous threshold after the function is done
    let previous = unsafe { __riscv_slic_raise_threshold(priority) };
    f();
    if let Ok(prev) = previous {
        // SAFETY: we restore the previous threshold after the function is done
        unsafe { __riscv_slic_set_threshold(prev) };
    }
}

/// Runs a function that takes a shared resource with a priority ceiling.
/// This function returns the return value of the target function.
///
/// # Safety
///
/// Input argument `ptr` must be a valid pointer to a shared resource.
#[inline]
pub unsafe fn lock<F, T, R>(ptr: *mut T, ceiling: u8, f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    // SAFETY: we restore the previous threshold after the function is done
    let previous = unsafe { __riscv_slic_raise_threshold(ceiling) };
    // SAFETY: provided that caller respects the safety requirements, this is safe
    let r = f(&mut *ptr);
    if let Ok(prev) = previous {
        // SAFETY: we restore the previous threshold after the function is done
        unsafe { __riscv_slic_set_threshold(prev) };
    }
    r
}
