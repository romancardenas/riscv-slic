use crate::input::CodegenInput;
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

fn swi_handler_signature() -> TokenStream {
    match () {
        #[cfg(feature = "msoft")]
        () => "MachineSoft".parse().unwrap(),
        #[cfg(feature = "ssoft")]
        () => "SupervisorSoft".parse().unwrap(),
    }
}

/// Creates the SLIC module with the proper interrupt sources.
pub fn swi_mod(input: &CodegenInput) -> TokenStream {
    let swi_handlers = &input.swi_handlers;
    let n_interrupts = swi_handlers.len();
    let swi_enums = interrupts_enum(swi_handlers);
    let swi_handler_signature = swi_handler_signature();

    quote!(
        /// Enumeration of software interrupts
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[repr(u16)]
        pub enum Interrupt {
            #(#swi_enums),*
        }

        unsafe impl riscv_slic::InterruptNumber for Interrupt {
            const MAX_INTERRUPT_NUMBER: u16 = #n_interrupts as u16 - 1;

            fn number(self) -> u16 {
                self as _
            }

            fn try_from(value: u16) -> Result<Self, u16> {
                if value > Self::MAX_INTERRUPT_NUMBER {
                    Err(value)
                } else {
                    // SAFETY: the value is less than the maximum interrupt number
                    Ok(unsafe { core::mem::transmute(value) })
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
        pub unsafe fn #swi_handler_signature() {
            export_swi_clear(); // We clear the software interrupt flag to allow nested interrupts
            riscv_slic::nested_isr(|| {
                while let Some((priority, interrupt)) = __SLIC.pop() {
                    riscv_slic::run(priority, || __SOFTWARE_INTERRUPTS[interrupt as usize]());
                }
            });
        }
    )
}
