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
        /// Triggers an environment call exception
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_swi_pend() {
            riscv_slic::nested(|| { riscv_slic::riscv::asm::ecall(); });
        }

        /// Increments the machine exception program counter by 4
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_swi_unpend() {
            let mepc = riscv_slic::riscv::register::mepc::read();
            riscv_slic::riscv::register::mepc::write(mepc + 4);
        }
    }
}
