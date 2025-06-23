use crate::input::SwiAttr;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Error, Ident, Path, Result,
    parse::{Parse, ParseStream},
};

pub struct ExportBackendInput {
    /// The identifier for the exception, defaults to `MachineEnvCall`
    exception: Ident,
}

impl Default for ExportBackendInput {
    fn default() -> Self {
        Self {
            exception: Ident::new("MachineEnvCall", Span::call_site()),
        }
    }
}

impl Parse for ExportBackendInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut exception = None;

        let content;
        syn::bracketed!(content in input);
        while !content.is_empty() {
            let ident: Ident = content.parse()?;
            match ident.to_string().as_str() {
                "exception" => {
                    if exception.is_some() {
                        return Err(Error::new(ident.span(), "duplicate identifier"));
                    }
                    content.parse::<syn::Token![=]>()?; // consume the '='
                    exception = Some(content.parse()?);
                }
                _ => return Err(Error::new(ident.span(), "invalid identifier")),
            }
            if !content.is_empty() {
                content.parse::<syn::Token![,]>()?; // consume the ',' between identifiers
            }
        }

        Ok(Self {
            exception: exception.unwrap_or_else(|| Ident::new("MachineEnvCall", input.span())),
        })
    }
}

pub fn export_swi_handler_attribute(pac: &Path, backend: &ExportBackendInput) -> TokenStream {
    let exception = &backend.exception;
    quote! {
        #[::riscv_rt::exception(#pac::interrupt::Exception::#exception)]
    }
}

pub fn export_quote(input: &SwiAttr) -> TokenStream {
    let slic = &input.slic;
    quote! {
        /// Triggers an environment call exception
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_swi_pend() {
            #slic::nested(|| { #slic::riscv::asm::ecall(); });
        }

        /// Increments the machine exception program counter by 4
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_swi_unpend() {
            let mepc = #slic::riscv::register::mepc::read();
            #slic::riscv::register::mepc::write(mepc + 4);
        }
    }
}
