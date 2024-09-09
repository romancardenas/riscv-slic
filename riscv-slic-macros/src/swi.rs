use crate::input::CodegenInput;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Path;

/// Helper function for generating the interrupt enums. It assigns a number to each source.
fn interrupts_enum(input: &[Ident]) -> Vec<TokenStream> {
    input
        .iter()
        .enumerate()
        .map(|(i, interrupt)| format!("{interrupt} = {i}").parse().unwrap())
        .collect()
}

fn swi_handler_attribute(pac: &Path) -> TokenStream {
    match () {
        #[cfg(feature = "mecall-backend")]
        () => quote! {
            #[riscv_rt::exception(#pac::interrupt::Exception::MachineEnvCall)]
        },
        #[cfg(feature = "msoft")]
        () => quote! {
            #[riscv_rt::core_interrupt(#pac::interrupt::CoreInterrupt::MachineSoft)]
        },
        #[cfg(feature = "ssoft")]
        () => quote! {
            #[riscv_rt::core_interrupt(#pac::interrupt::CoreInterrupt::SupervisorSoft)]
        },
    }
}

/// Creates the SLIC module with the proper interrupt sources.
pub fn swi_mod(input: &CodegenInput) -> TokenStream {
    let mut res = Vec::new();

    let swi_handlers = &input.swi_handlers;
    let n_interrupts = swi_handlers.len();
    let swi_enums = interrupts_enum(swi_handlers);
    let swi_handler_attribute = swi_handler_attribute(&input.pac);

    if n_interrupts > 0 {
        res.push(quote!(
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            #[doc(hidden)]
            #[repr(u16)]
            pub enum SoftwareInterrupt {
                #(#swi_enums),*
            }

            unsafe impl riscv_slic::InterruptNumber for SoftwareInterrupt {
                const MAX_INTERRUPT_NUMBER: u16 = #n_interrupts as u16 - 1;

                #[inline]
                fn number(self) -> u16 {
                    self as u16
                }

                #[inline]
                fn from_number(value: u16) -> Result<Self, u16> {
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
        ));
    }
    res.push(quote!(
        /// Array of software interrupt handlers in the order of the `Interrupt` enum.
        static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); #n_interrupts] = [
            #(#swi_handlers),*
        ];

        /// The static SLIC instance
        static mut __SLIC: riscv_slic::MutexSLIC<#n_interrupts> = riscv_slic::new_slic();

        /// Software interrupt handler to be used with the SLIC.
        #swi_handler_attribute
        unsafe fn riscv_slic_swi_handler() {
            __riscv_slic_swi_unpend();
            // We nest the handler to let other interrupts trigger
            riscv_slic::nested(|| unsafe { __riscv_slic_pop() });
        }
    ));
    quote!(#(#res)*)
}
