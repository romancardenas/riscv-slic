use proc_macro::TokenStream;
use quote::quote;

mod api;
mod export;
mod input;
mod swi;

// Ex. codegen!(pac = <pac crate>, swi = [list, of, software, interrupts], backend = <backend-specific configuration>)
#[proc_macro]
pub fn codegen(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as input::CodegenInput);
    let slic = &input.slic;
    let pac = &input.pac;

    let api_code = api::api_mod();

    let swi_export = export::export_quote(&input);
    let swi_code = swi::swi_mod(&input);

    quote! {
        /// The RISC-V SLIC module
        pub mod slic {
            use super::#pac;
            use #slic::{self, *};

            #api_code

            #swi_export
            #swi_code
        }
    }
    .into()
}
