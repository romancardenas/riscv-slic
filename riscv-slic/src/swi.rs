use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Helper function for generating the interrupt enums. It assigns a number to each source.
fn interrupts_enum(input: &[Ident]) -> Vec<TokenStream> {
    input
        .iter()
        .enumerate()
        .map(|(i, interrupt)| format!("{interrupt} = {i}").parse().unwrap())
        .collect()
}

/// Helper function for the [`TryFrom`] trait from [`u16`] to software interrupts.
fn u16_to_swi(input: &[Ident]) -> Vec<TokenStream> {
    input
        .iter()
        .enumerate()
        .map(|(i, interrupt)| format!("{i} => Ok(Self::{interrupt}),").parse().unwrap())
        .collect()
}

/// Creates the SLIC module with the proper interrupt sources.
pub fn swi_mod(swi_handlers: &[Ident]) -> TokenStream {
    let n_interrupts = swi_handlers.len();
    let swi_enums = interrupts_enum(&swi_handlers);
    let u16_matches = u16_to_swi(&swi_handlers);

    quote!(
        /// Enumeration of software interrupts
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[repr(u16)]
        pub enum Interrupt {
            #(#swi_enums),*
        }

        impl TryFrom<u16> for Interrupt {
            type Error = u16;

            #[inline]
            fn try_from(value: u16) -> Result<Self, Self::Error> {
                match value {
                    #(#u16_matches)*
                    _ => Err(value),
                }
            }
        }

        extern "C" {
            #(fn #swi_handlers ();)*
        }

        #[no_mangle]
        pub static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); #n_interrupts] = [
            #(#swi_handlers),*
        ];

        pub static mut __SLIC: SLIC = SLIC::new();

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe fn MachineSoft() {
            swi_clear(); // We clear it at the beginning to allow nested interrupts
            while let Some((priority, interrupt)) = __SLIC.pop() {
                run(priority, || __SOFTWARE_INTERRUPTS[interrupt as usize]());
            }
        }
    )
}
