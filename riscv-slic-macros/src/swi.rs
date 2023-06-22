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
    let swi_enums = interrupts_enum(swi_handlers);
    let u16_matches = u16_to_swi(swi_handlers);

    quote!(
        /// Enumeration of software interrupts
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[repr(u16)]
        pub enum Interrupt {
            #(#swi_enums),*
        }

        unsafe impl riscv_slic::swi::InterruptNumber for Interrupt {
            const MAX_INTERRUPT_NUMBER: u16 = #n_interrupts as u16 - 1;

            fn number(self) -> u16 {
                self as _
            }

            fn try_from(value: u16) -> Result<Self, u16> {
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

        pub static mut __SLIC: riscv_slic::SLIC<#n_interrupts> = riscv_slic::SLIC::new();

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe fn MachineSoft() {
            swi_clear(); // We clear the software interrupt flag to allow nested interrupts
            riscv_slic::nested_isr(|| {
                while let Some((priority, interrupt)) = __SLIC.pop() {
                    riscv_slic::run(priority, || __SOFTWARE_INTERRUPTS[interrupt as usize]());
                }
            });
        }
    )
}
