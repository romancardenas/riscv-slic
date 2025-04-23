use crate::input::{CodegenInput, SwiAttr};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Error, Path, Result,
    parse::{Parse, ParseStream},
};

pub struct ExportBackendInput();

impl Parse for ExportBackendInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Err(Error::new(
            input.span(),
            "This backend does not require any input",
        ))
    }
}

pub fn export_swi_handler_attribute(pac: &Path) -> TokenStream {
    quote! {
        #[::riscv_rt::core_interrupt(#pac::interrupt::CoreInterrupt::SupervisorSoft)]
    }
}

pub fn export_quote(input: &SwiAttr) -> TokenStream {
    let slic = &input.slic;
    quote! {
        /// Triggers a supervisor software interrupt via the `SIP` register.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_swi_pend() {
            #slic::riscv::register::sip::set_ssoft();
        }

        /// Clears the Supervisor Software Interrupt Pending bit in the `SIP` register.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_swi_unpend() {
            #slic::riscv::register::sip::clear_ssoft();
        }
    }
}
