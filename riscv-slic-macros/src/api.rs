use proc_macro2::TokenStream;
use quote::quote;

pub fn api_mod() -> TokenStream {
    quote!(
        use riscv_slic::InterruptNumber; // expose the InterruptNumber trait

        /// Returns the current priority threshold of the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_get_threshold() -> u8 {
            __SLIC.get_threshold()
        }

        /// Sets the priority threshold of the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_set_threshold(thresh: u8) {
            __SLIC.set_threshold(thresh);
            // check if we need to trigger a software interrupt after changing the threshold
            if __SLIC.is_ready() {
                __riscv_slic_swi_pend();
            }
        }

        /// Returns the interrupt priority of a given software interrupt source.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_get_priority(interrupt: u16) -> u8 {
            __SLIC.get_priority(interrupt)
        }

        /// Sets the interrupt priority of a given software interrupt source in the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_set_priority(interrupt: u16, priority: u8) {
            __SLIC.set_priority(interrupt, priority);
        }

        /// Marks a software interrupt as pending.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_pend(interrupt: u16) {
            __SLIC.pend(interrupt);
            if __SLIC.is_ready() {
                __riscv_slic_swi_pend();
            }
        }

        /// Polls the SLIC for pending software interrupts and runs them.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_run() {
            while let Some((priority, interrupt)) = __SLIC.pop() {
                riscv_slic::run(priority, || __SOFTWARE_INTERRUPTS[interrupt as usize]());
            }
        }
    )
}
