use proc_macro2::{Ident, TokenStream};
use quote::quote;

#[cfg(not(any(feature = "exti-plic")))]
pub(crate) fn export_exti(pac: &Ident) -> TokenStream {
    quote! {}
}

#[cfg(feature = "exti-plic")]
pub(crate) fn export_exti(pac: &Ident) -> TokenStream {
    quote! {
        #[inline(always)]
        fn exti_claim() -> Option<#pac::Interrupt> {
            #pac::PLIC::claim()
        }

        #[inline(always)]
        fn exti_complete(exti: #pac::Interrupt) {
            #pac::PLIC::complete(exti);
        }
    }
}

#[cfg(feature = "swi-clint")]
pub(crate) fn export_swi(pac: &Ident) -> TokenStream {
    quote! {
        /// Triggers a machine software interrupt via the CLINT peripheral
        pub unsafe fn swi_set() {
            let clint = #pac::Peripherals::steal().CLINT;
            clint.msip.write(|w| w.bits(0x01));
        }

        /// Clears the Machine Software Interrupt Pending bit via the CLINT peripheral
        pub unsafe fn swi_clear() {
            let clint = #pac::Peripherals::steal().CLINT;
            clint.msip.write(|w| w.bits(0x00));
        }
    }
}
