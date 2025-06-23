use crate::input::{CodegenInput, SwiAttr};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Error, Path, Result,
    parse::{Parse, ParseStream},
};

pub struct ExportBackendInput {
    /// The identifier for the core interrupt, defaults to `SupervisorSoft`
    core_interrupt: Ident,
}

impl Default for ExportBackendInput {
    fn default() -> Self {
        Self {
            core_interrupt: Ident::new("SupervisorSoft", Span::call_site()),
        }
    }
}

impl Parse for ExportBackendInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut core_interrupt = None;

        let content;
        syn::bracketed!(content in input);
        while !content.is_empty() {
            let ident: Ident = content.parse()?;
            match ident.to_string().as_str() {
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
            core_interrupt: core_interrupt
                .unwrap_or_else(|| Ident::new("SupervisorSoft", input.span())),
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
