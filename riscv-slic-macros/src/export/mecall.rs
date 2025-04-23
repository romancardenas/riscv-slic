use crate::input::SwiAttr;
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
        #[::riscv_rt::exception(#pac::interrupt::Exception::MachineEnvCall)]
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
