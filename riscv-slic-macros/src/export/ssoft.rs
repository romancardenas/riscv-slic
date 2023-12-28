use crate::input::CodegenInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Error, Result,
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

pub fn export_quote(_input: &CodegenInput) -> TokenStream {
    quote! {
        /// Triggers a supervisor software interrupt via the `SIP` register.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_swi_pend() {
            riscv_slic::riscv::register::sip::set_ssoft();
        }

        /// Clears the Supervisor Software Interrupt Pending bit in the `SIP` register.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_swi_unpend() {
            riscv_slic::riscv::register::sip::clear_ssoft();
        }
    }
}
