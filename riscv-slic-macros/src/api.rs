use proc_macro2::TokenStream;
use quote::quote;

pub fn api_mod() -> TokenStream {
    quote!(
        use riscv_slic::InterruptNumber; // expose the InterruptNumber trait

        /// Clears all interrupt flags to avoid interruptions of SLIC and HW controller.
        #[inline]
        #[no_mangle]
        pub unsafe fn __slic_clear() {
            export_swi_clear();
        }

        /// Returns the current priority threshold of the SLIC.
        #[inline]
        #[no_mangle]
        pub unsafe fn __slic_get_threshold() -> u8 {
            __SLIC.get_threshold()
        }

        /// Sets the priority threshold of the external interrupt controller and the SLIC.
        #[inline]
        #[no_mangle]
        pub unsafe fn __slic_set_threshold(thresh: u8) {
            __SLIC.set_threshold(thresh);
            // check if we need to trigger a software interrupt after changing the threshold
            if __SLIC.is_ready() {
                export_swi_set();
            }
        }

        /// Returns the interrupt priority of a given software interrupt source.
        #[inline]
        #[no_mangle]
        pub unsafe fn __slic_get_priority(interrupt: u16) -> u8 {
            __SLIC.get_priority(interrupt)
        }

        /// Sets the interrupt priority of a given software interrupt
        /// source in the external interrupt controller and the SLIC.
        #[inline]
        #[no_mangle]
        pub unsafe fn __slic_set_priority(interrupt: u16, priority: u8) {
            __SLIC.set_priority(interrupt, priority);
        }

        /// Marks a software interrupt as pending.
        #[inline]
        #[no_mangle]
        pub unsafe fn __slic_pend(interrupt: u16) {
            __SLIC.pend(interrupt);
            if __SLIC.is_ready() {
                export_swi_set();
            }
        }
    )
}
