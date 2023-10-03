use proc_macro::TokenStream;
use quote::quote;

mod api;
mod export;
mod input;
mod swi;

// Ex. codegen!(pac, [HW1, HW2], [SW1, SW2])
// Ex. codegen!(e310x, [GPIO1, RTC], [Task1, Task2])
#[proc_macro]
pub fn codegen(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as input::CodegenInput);

    let api_code = api::api_mod();

    let swi_export = export::export_quote(&input);
    let swi_code = swi::swi_mod(&input);

    quote! {
        pub mod slic {
            use super::riscv_slic;

            #api_code

            #swi_export
            #swi_code
        }
    }
    .into()
}
