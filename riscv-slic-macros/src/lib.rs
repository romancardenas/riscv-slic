use proc_macro::TokenStream;
use quote::quote;

mod api;
mod export;
mod input;
mod swi;

/// Generates all the necessary code for creating a SLIC struct and its associated functions.
///
/// # Usage
///
/// ```ignore
/// codegen!([slic = riscv_slic,] pac = <pac::crate>, swi = [list, of, software, interrupts][, backend = <backend-specific configuration>]);
/// ```
///
/// # Arguments
///
/// * `slic` - Path to the SLIC crate. This is optional and defaults to `riscv_slic`.
/// * `pac` - Path to the peripheral access crate for the target device. This is required.
/// * `swi` - A list of software interrupts to handle. This is required, but can be an empty list.
/// * `backend` - The backend-specific configuration. Depending on the backend, this may be required or optional.
///
/// # Backend-specific configuration
///
/// ## `clint-backend`
///
/// * `hart_id` - The identifier of the HART whose software interrupt should be triggered.
///
/// ### Example
///
/// ```ignore
/// codegen!(pac = pac, swi = [list, of, software, interrupts], backend = { hart_id = H0 });
/// ```
///
/// ## `mecall-backend`
///
/// This backend does not require any configuration.
///
/// ### Example
///
/// ```ignore
/// codegen!(pac = pac, swi = [list, of, software, interrupts]);
/// ```
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
