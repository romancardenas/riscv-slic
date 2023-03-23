use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;

/// Helper function for generating the interrupt enums. It assigns a number to each source.
fn interrupts_enum(input: &[Ident]) -> Vec<TokenStream2> {
    input
        .iter()
        .enumerate()
        .map(|(i, interrupt)| format!("{interrupt} = {i}").parse().unwrap())
        .collect()
}

/// Helper function for the try_into method of interrupts. It retrieves the number of each source.
fn interrupts_into(input: &[Ident]) -> Vec<TokenStream2> {
    input
        .iter()
        .enumerate()
        .map(|(i, interrupt)| format!("{i} => Ok(Self::{interrupt}),").parse().unwrap())
        .collect()
}

#[proc_macro]
pub fn codegen(input: TokenStream) -> TokenStream {
    let input: TokenStream2 = input.into();
    let mut interrupts_ident: Vec<Ident> = Vec::new();

    // INPUT TOKEN STREAM PARSING
    // Even tokens must be interrupt source identifiers, and odd tokens must be commas
    for (i, token) in input.into_iter().enumerate() {
        if i % 2 == 0 {
            if let TokenTree::Ident(ident) = token {
                interrupts_ident.push(ident);
                continue;
            }
            return quote!(
                use invalid_input::input_must_be_interrupt_sources::separated_by_comma;
            )
            .into();
        } else {
            if let TokenTree::Punct(punct) = &token {
                if punct.as_char() == ',' {
                    continue;
                }
            }
            return quote!(
                use invalid_input::input_must_be_interrupt_sources::separated_by_comma;
            )
            .into();
        }
    }
    let n_interrupts: usize = interrupts_ident.len();
    // There must be at least one interrupt source
    if n_interrupts == 0 {
        return quote!(
            use invalid_input::you_must_define_at_least::one_interrupt_source::separated_by_comma;
        )
        .into();
    }

    let interrupts_enum = interrupts_enum(&interrupts_ident);
    let interrupts_into = interrupts_into(&interrupts_ident);

    quote! {
        pub mod slic {
            #[repr(u16)]
            pub enum Interrupt {
                #(#interrupts_enum),*
            }

            impl Interrupt {
                #[inline]
                pub fn try_from(value: u16) -> Result<Self, u16> {
                    match value {
                        #(#interrupts_into)*
                        _ => Err(value),
                    }
                }
            }

            extern "C" {
                #(fn #interrupts_ident ();)*
            }

            #[no_mangle]
            pub static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); #n_interrupts] = [
                #(#interrupts_ident),*
            ];

            #[no_mangle]
            pub static mut __SLIC: riscv_slic::SLIC<#n_interrupts> = riscv_slic::SLIC::new();

            #[no_mangle]
            pub unsafe fn SoftwareExternal() {
                riscv_slic::export::clear_interrupt();
                __SLIC.pop(&__SOFTWARE_INTERRUPTS);
            }
        }
    }
    .into()
}
