use crate::input::CodegenInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, Result, Token,
};

pub struct ExportBackendInput {
    /// The identifier of the MSIP register in the CLINT peripheral
    hart_id: Ident,
}

impl Parse for ExportBackendInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut hart_id = None;

        let content;
        syn::bracketed!(content in input);
        while !content.is_empty() {
            let ident: Ident = content.parse()?;
            match ident.to_string().as_str() {
                "hart_id" => {
                    if hart_id.is_some() {
                        return Err(Error::new(ident.span(), "duplicate identifier"));
                    }
                    content.parse::<Token![=]>()?; // consume the '='
                    hart_id = Some(content.parse()?);
                }
                _ => return Err(Error::new(ident.span(), "invalid identifier")),
            }
            if !content.is_empty() {
                content.parse::<Token![,]>()?; // consume the ',' between identifiers
            }
        }

        Ok(Self {
            hart_id: hart_id.ok_or_else(|| Error::new(input.span(), "missing identifier"))?,
        })
    }
}

pub fn export_quote(input: &CodegenInput) -> TokenStream {
    let pac = &input.pac;
    let backend = input.backend.as_ref().unwrap();
    let hart_id = &backend.hart_id;
    quote! {
        /// Triggers a machine software interrupt via the CLINT peripheral.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_swi_pend() {
            let msip = #pac::CLINT::mswi().msip(#pac::interrupt::Hart::#hart_id);
            msip.pend();
        }

        /// Clears the Machine Software Interrupt Pending bit via the CLINT peripheral.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_swi_unpend() {
            let msip = #pac::CLINT::mswi().msip(#pac::interrupt::Hart::#hart_id);
            msip.unpend();
        }
    }
}
