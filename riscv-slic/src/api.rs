extern "C" {
    fn __slic_clear();
    fn __slic_set_threshold(priority: u8);
    fn __slic_get_threshold() -> u8;
    fn __slic_get_priority(interrupt: u16) -> u8;
    fn __slic_set_priority(interrupt: u16, priority: u8);
    fn __slic_pend(interrupt: u16);
}

/// Clears all interrupt flags to avoid interruptions.
#[inline(always)]
pub unsafe fn clear_interrupts() {
    riscv::register::mstatus::clear_mie();
    riscv::register::mie::clear_mext();
    riscv::register::mie::clear_msoft();
    __slic_clear();
    set_threshold(u8::MAX);
}

/// Sets all the interrupt flags to allow external and software interrupts.
/// It also sets the interrupt threshold to 0 (i.e., accept all interrupts).
#[inline(always)]
pub unsafe fn set_interrupts() {
    set_threshold(0);
    riscv::register::mstatus::set_mie();
    riscv::register::mie::set_mext();
    riscv::register::mie::set_msoft();
}

/// Stabilized API for changing the threshold of the SLIC.
///
/// # Safety
///
/// Changing the priority threshold may break mask-based critical sections.
#[inline(always)]
pub unsafe fn set_threshold(priority: u8) {
    __slic_set_threshold(priority);
}

/// Stabilized API for getting the current threshold of the SLIC.
#[inline(always)]
pub fn get_threshold() -> u8 {
    // SAFETY: this read has no side effects.
    unsafe { __slic_get_threshold() }
}

/// Stabilized API for getting the priority of a given software interrupt source.
#[inline(always)]
pub fn get_priority<I: crate::swi::InterruptNumber>(interrupt: I) -> u8 {
    // SAFETY: this read has no side effects.
    unsafe { __slic_get_priority(interrupt.number()) }
}

/// Stabilized API for setting the priority of a software interrupt of the SLIC.
///
/// # Safety
///
/// Changing the priority of an interrupt may break mask-based critical sections.
#[inline(always)]
pub unsafe fn set_priority<I: crate::swi::InterruptNumber>(interrupt: I, priority: u8) {
    __slic_set_priority(interrupt.number(), priority);
}

/// Stabilized API for pending a software interrupt on the SLIC.
#[inline(always)]
pub fn pend<I: crate::swi::InterruptNumber>(interrupt: I) {
    // SAFETY: TODO
    unsafe { __slic_pend(interrupt.number()) };
}

/// Runs a function with priority mask.
///
/// # Safety
///
/// If new priority is less than current priority, priority inversion may occur.
#[inline(always)]
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
#[inline(always)]
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
