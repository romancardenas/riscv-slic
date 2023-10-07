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
        /// Triggers a machine software interrupt via the CLINT peripheral
        #[inline]
        pub unsafe fn export_swi_set() {
            riscv_slic::riscv::register::mip::set_ssoft();
        }

        /// Clears the Machine Software Interrupt Pending bit via the CLINT peripheral
        #[inline]
        pub unsafe fn export_swi_clear() {
            riscv_slic::riscv::register::mip::clear_ssoft();
        }
    }
}
