use proc_macro2::TokenStream;
use quote::quote;

pub fn extern_mod() -> TokenStream {
    quote!(
        use riscv_slic::swi::InterruptNumber;
        /// Returns the current priority threshold of the SLIC.
        #[inline(always)]
        #[no_mangle]
        pub unsafe fn __slic_get_threshold() -> u8 {
            __SLIC.get_threshold()
        }

        /// Sets the priority threshold of the external interrupt controller and the SLIC.
        #[inline(always)]
        #[no_mangle]
        pub unsafe fn __slic_set_threshold(thresh: u8) {
            exti_set_threshold(thresh);
            __SLIC.set_threshold(thresh);
        }

        /// Marks a software interrupt as pending.
        #[inline(always)]
        #[no_mangle]
        pub unsafe fn __slic_pend(interrupt: u16) {
            let interrupt: Interrupt = InterruptNumber::try_from(interrupt).unwrap();
            __SLIC.pend(interrupt);
            if __SLIC.is_ready() {
                swi_set();
            }
        }
    )
}
