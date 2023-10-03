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
        #[inline]
        pub unsafe fn export_swi_set() {
            todo!();
        }

        /// Clears a software interrupt
        #[inline]
        pub unsafe fn export_swi_clear() {
            todo!();
        }
    }
}
