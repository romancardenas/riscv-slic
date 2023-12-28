use crate::input::CodegenInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Error, Result,
};

pub struct BackendInput {
    // Define your backend-specific input here
}

impl Parse for BackendInput {
    fn parse(input: ParseStream) -> Result<Self> {
        todo!("define how to parse your backend-specific input")
    }
}

pub fn export_quote(input: &CodegenInput) -> TokenStream {
    quote! {
        /// Triggers a software interrupt
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_swi_pend() {
            todo!("define how to trigger a software interrupt");
        }

        /// Clears a software interrupt
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_swi_unpend() {
            todo!("define how to clear a software interrupt");
        }
    }
}
