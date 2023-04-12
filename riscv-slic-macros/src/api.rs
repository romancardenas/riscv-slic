use proc_macro2::TokenStream;
use quote::quote;

/// Creates the SLIC module with the proper interrupt sources.
pub fn api_mod() -> TokenStream {
    quote!(
        /// Clears all interrupt flags to avoid interruptions.
        #[inline(always)]
        pub unsafe fn clear_interrupts() {
            riscv_slic::riscv::register::mstatus::clear_mie();
            riscv_slic::riscv::register::mie::clear_mext();
            riscv_slic::riscv::register::mie::clear_msoft();
            exti_clear();
            swi_clear();
            riscv_slic::set_threshold(u8::MAX);
        }

        /// Sets all the interrupt flags to allow external and software interrupts.
        /// It also sets the interrup threshold to 0 (i.e., accept all interrupts).
        #[inline(always)]
        pub unsafe fn set_interrupts() {
            riscv_slic::set_threshold(0);
            riscv_slic::riscv::register::mstatus::set_mie();
            riscv_slic::riscv::register::mie::set_mext();
            riscv_slic::riscv::register::mie::set_msoft();
        }

        /// Returns the interrupt priority of a given software interrupt source.
        #[inline(always)]
        pub unsafe fn get_priority(interrupt: Interrupt) -> u8 {
            __SLIC.get_priority(interrupt)
        }

        /// Sets the interrupt priority of a given software interrupt
        /// source in the external interrupt controller and the SLIC.
        #[inline(always)]
        pub unsafe fn set_priority(interrupt: Interrupt, priority: u8) {
            __SLIC.set_priority(interrupt, priority);
            if let Ok(exti) = interrupt.try_into() {
                exti_set_priority(exti, priority);
            }
        }

        /// Runs a function with priority mask.
        ///
        /// # Safety
        ///
        /// If new priority is less than current priority, priority inversion may occur.
        #[inline(always)]
        pub unsafe fn run<F: FnOnce()>(priority: u8, f: F) {
            let current = riscv_slic::get_threshold();
            riscv_slic::set_threshold(priority);
            f();
            riscv_slic::set_threshold(current);
        }

        /// Runs a function that takes a shared resource with a priority ceiling.
        /// This function returns the return value of the target function.
        #[inline(always)]
        pub unsafe fn lock<F, T, R>(ptr: *mut T, ceiling: u8, f: F) -> R
        where
            F: FnOnce(&mut T) -> R,
        {
            let current = riscv_slic::get_threshold();
            riscv_slic::set_threshold(ceiling);
            let r = f(&mut *ptr);
            riscv_slic::set_threshold(current);
            r
        }
    )
}
