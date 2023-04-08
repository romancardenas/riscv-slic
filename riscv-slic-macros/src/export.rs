use proc_macro2::{Ident, TokenStream};
use quote::quote;

#[cfg(not(any(feature = "exti-plic")))]
pub(crate) fn export_exti(pac: &Ident) -> TokenStream {
    quote! {
        /// Returns a pending external interrupt.
        /// If no external interrupts are pending, it returns `None`.
        #[inline(always)]
        fn exti_claim() -> Option<#pac::Interrupt> {
            None
        }

        #[inline(always)]
        unsafe fn exti_clear() {}

        /// Marks a pending external interrupt as complete.
        /// If the interrupt was not pending, it silently ignores it.
        #[inline(always)]
        fn exti_complete(exti: #pac::Interrupt) {}

        /// Sets the priority threshold of the external interrupt controller.
        #[inline(always)]
        fn exti_set_threshold(threshold: u8) {}

        /// Sets the priority threshold of the external interrupt controller.
        #[inline(always)]
        fn exti_set_priority(interrupt: #pac::Interrupt, priority: u8) {}
    }
}

fn common_exti(pac: &Ident) -> TokenStream {
    quote! {
        use riscv_slic::exti::PriorityNumber;

        /// Converts an `u8` to the corresponding priority level.
        /// If conversion fails, it returns the highest available priority level.
        #[inline(always)]
        fn saturated_priority(mut priority: u8) -> #pac::Priority {
            if priority > #pac::Priority::MAX_PRIORITY_NUMBER {
                priority = #pac::Priority::MAX_PRIORITY_NUMBER;
            }
            #pac::Priority::try_from(priority).unwrap()
        }
    }
}

#[cfg(feature = "exti-plic")]
pub(crate) fn export_exti(pac: &Ident) -> TokenStream {
    let common = common_exti(pac);
    quote! {
        #common

        #[inline(always)]
        unsafe fn exti_clear() {
            let mut plic = #pac::Peripherals::steal().PLIC;
            plic.reset();
        }

        /// Returns the next pending external interrupt according to the PLIC.
        /// If no external interrupts are pending, it returns `None`.
        #[inline(always)]
        fn exti_claim() -> Option<#pac::Interrupt> {
            #pac::PLIC::claim()
        }

        /// Notifies the PLIC that a pending external interrupt as complete.
        /// If the interrupt was not pending, it silently ignores it.
        #[inline(always)]
        fn exti_complete(exti: #pac::Interrupt) {
            #pac::PLIC::complete(exti);
        }

        /// Sets the PLIC threshold to the desired value. If threshold is higher than
        /// the highest priority, it sets the threshold to the highest possible value.
        #[inline(always)]
        unsafe fn exti_set_threshold(threshold: u8) {
            let mut plic = #pac::Peripherals::steal().PLIC;
            plic.set_threshold(saturated_priority(threshold));
        }

        /// Enables the PLIC interrupt source and sets its priority to the desired value.
        /// If priority is higher than the highest priority, it sets it to the highest possible value.
        #[inline(always)]
        unsafe fn exti_set_priority(interrupt: #pac::Interrupt, priority: u8) {
            let mut plic = #pac::Peripherals::steal().PLIC;
            plic.enable_interrupt(interrupt);
            plic.set_priority(interrupt, saturated_priority(priority));
        }
    }
}

#[cfg(feature = "swi-clint")]
pub(crate) fn export_swi(pac: &Ident) -> TokenStream {
    quote! {
        /// Triggers a machine software interrupt via the CLINT peripheral
        #[inline(always)]
        pub unsafe fn swi_set() {
            let clint = #pac::Peripherals::steal().CLINT;
            clint.msip.write(|w| w.bits(0x01));
        }

        /// Clears the Machine Software Interrupt Pending bit via the CLINT peripheral
        #[inline(always)]
        pub unsafe fn swi_clear() {
            let clint = #pac::Peripherals::steal().CLINT;
            clint.msip.write(|w| w.bits(0x00));
        }
    }
}
