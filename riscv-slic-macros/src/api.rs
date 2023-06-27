use proc_macro2::TokenStream;
use quote::quote;

pub fn api_mod() -> TokenStream {
    quote!(
        use riscv_slic::swi::InterruptNumber;

        /// Clears all interrupt flags to avoid interruptions of SLIC and HW controller.
        #[inline(always)]
        #[no_mangle]
        pub unsafe fn __slic_clear() {
            exti_clear();
            swi_clear();
        }

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
            // check if we need to trigger a software interrupt after changing the threshold
            if __SLIC.is_ready() {
                swi_set();
            }
        }

        /// Returns the interrupt priority of a given software interrupt source.
        #[inline(always)]
        #[no_mangle]
        pub unsafe fn __slic_get_priority(interrupt: u16) -> u8 {
            let interrupt: Interrupt = InterruptNumber::try_from(interrupt).unwrap();
            __SLIC.get_priority(interrupt)
        }

        /// Sets the interrupt priority of a given software interrupt
        /// source in the external interrupt controller and the SLIC.
        #[inline(always)]
        #[no_mangle]
        pub unsafe fn __slic_set_priority(interrupt: u16, priority: u8) {
            let interrupt: Interrupt = InterruptNumber::try_from(interrupt).unwrap();
            __SLIC.set_priority(interrupt, priority);
            if let Ok(exti) = interrupt.try_into() {
                exti_set_priority(exti, priority);
            }
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
