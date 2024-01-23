use proc_macro2::TokenStream;
use quote::quote;

pub fn api_mod() -> TokenStream {
    quote!(
        /// Returns the current priority threshold of the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_get_threshold() -> u8 {
            critical_section::with(|cs| __SLIC.borrow_ref(cs).get_threshold())
        }

        /// Sets the priority threshold of the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_set_threshold(thresh: u8) {
            critical_section::with(|cs| {
                let mut slic = __SLIC.borrow_ref_mut(cs);
                slic.set_threshold(thresh);
                if slic.is_ready() {
                    __riscv_slic_swi_pend();
                }
            });
        }

        /// Returns the interrupt priority of a given software interrupt source.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_get_priority(interrupt: u16) -> u8 {
            critical_section::with(|cs| __SLIC.borrow_ref(cs).get_priority(interrupt))
        }

        /// Sets the interrupt priority of a given software interrupt source in the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_set_priority(interrupt: u16, priority: u8) {
            critical_section::with(|cs| {
                __SLIC.borrow_ref_mut(cs).set_priority(interrupt, priority)
            });
        }

        /// Marks a software interrupt as pending.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_pend(interrupt: u16) {
            critical_section::with(|cs| {
                let mut slic = __SLIC.borrow_ref_mut(cs);
                slic.pend(interrupt);
                if slic.is_ready() {
                    __riscv_slic_swi_pend();
                }
            });
        }

        /// Polls the SLIC for pending software interrupts and runs them.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_run() {
            if let Some((pri, int)) = critical_section::with(|cs| __SLIC.borrow_ref_mut(cs).pop()) {
                riscv_slic::run(pri, || __SOFTWARE_INTERRUPTS[int as usize]());
            }
        }
    )
}
