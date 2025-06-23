use crate::input::SwiAttr;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Error, Ident, Path, Result, Token,
    parse::{Parse, ParseStream},
};

pub struct ExportBackendInput {
    /// The identifier for the CLINT peripheral, defaults to `Clint`
    clint_id: Ident,
    /// The identifier for the core interrupt, defaults to `MachineSoft`
    core_interrupt: Ident,
}

impl Default for ExportBackendInput {
    fn default() -> Self {
        Self {
            clint_id: Ident::new("Clint", Span::call_site()),
            core_interrupt: Ident::new("MachineSoft", Span::call_site()),
        }
    }
}

impl Parse for ExportBackendInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut clint_id = None;
        let mut core_interrupt = None;

        let content;
        syn::bracketed!(content in input);
        while !content.is_empty() {
            let ident: Ident = content.parse()?;
            match ident.to_string().as_str() {
                "clint_id" => {
                    if clint_id.is_some() {
                        return Err(Error::new(ident.span(), "duplicate identifier"));
                    }
                    content.parse::<Token![=]>()?; // consume the '='
                    clint_id = Some(content.parse()?);
                }
                "core_interrupt" => {
                    if core_interrupt.is_some() {
                        return Err(Error::new(ident.span(), "duplicate identifier"));
                    }
                    content.parse::<Token![=]>()?; // consume the '='
                    core_interrupt = Some(content.parse()?);
                }
                _ => return Err(Error::new(ident.span(), "invalid identifier")),
            }
            if !content.is_empty() {
                content.parse::<Token![,]>()?; // consume the ',' between identifiers
            }
        }

        Ok(Self {
            clint_id: clint_id.unwrap_or(Ident::new("Clint", input.span())),
            core_interrupt: core_interrupt
                .unwrap_or_else(|| Ident::new("MachineSoft", input.span())),
        })
    }
}

pub fn export_swi_handler_attribute(pac: &Path, backend: &ExportBackendInput) -> TokenStream {
    let core_interrupt = &backend.core_interrupt;
    quote! {
        #[::riscv_rt::core_interrupt(#pac::interrupt::CoreInterrupt::#core_interrupt)]
    }
}

pub fn export_quote(input: &SwiAttr) -> TokenStream {
    let pac = &input.pac;
    let backend = &input.backend;
    let clint_id = &backend.clint_id;
    quote! {
        /// Triggers a machine software interrupt via the CLINT peripheral.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_swi_pend() {
            let clint = unsafe { #pac::#clint_id::steal() };
            let msip = clint.mswi().msip_mhartid();
            msip.pend();
        }

        /// Clears the Machine Software Interrupt Pending bit via the CLINT peripheral.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_swi_unpend() {
            let clint = unsafe { #pac::#clint_id::steal() };
            let msip = clint.mswi().msip_mhartid();
            msip.unpend();
        }
    }
}
