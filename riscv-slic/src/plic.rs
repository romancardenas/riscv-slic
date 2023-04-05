use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Helper function for the [`TryFrom`] trait from hardware interrupts to software interrupts.
fn hw_to_sw(pac: &Ident, input: &[Ident]) -> Vec<TokenStream> {
    input
        .iter()
        .map(|interrupt| {
            format!(
                "{}::Interrupt::{} => Ok(Interrupt::{}),",
                pac.to_string(),
                interrupt.to_string(),
                interrupt.to_string(),
            )
            .parse()
            .unwrap()
        })
        .collect()
}

fn hw_to_clear(input: &[Ident]) -> Vec<TokenStream> {
    input
        .iter()
        .map(|interrupt| format!("Clear{}", interrupt.to_string(),).parse().unwrap())
        .collect()
}

/// Creates the SLIC module with the proper interrupt sources.
pub fn hw_mod(pac: &Ident, hw_handlers: &[Ident]) -> TokenStream {
    let n_hw_interrupts = hw_handlers.len();
    if n_hw_interrupts == 0 {
        return quote!();
    }
    let hw_matches = hw_to_sw(pac, hw_handlers);
    let hw_clear = hw_to_clear(hw_handlers);

    quote! {
        impl TryFrom<#pac::Interrupt> for Interrupt {
            type Error = #pac::Interrupt;
            fn try_from(value: #pac::Interrupt) -> Result<Self, Self::Error> {
                match value {
                    #(#hw_matches)*
                    _ => Err(value),
                }
            }
        }

        extern "C" {
            #(fn #hw_clear ();)*
        }

        #[no_mangle]
        pub static __CLEAR_EXTERNAL_INTERRUPTS: [unsafe extern "C" fn(); #n_hw_interrupts] = [
            #(#hw_clear),*
        ];

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe fn MachineExternal() {
            if let Some(hw_interrupt) = #pac::PLIC::claim() {
                let sw_interrupt: Result<Interrupt, #pac::Interrupt> = hw_interrupt.try_into();
                match sw_interrupt {
                    Ok(sw_interrupt) => {
                        __CLEAR_EXTERNAL_INTERRUPTS[sw_interrupt as usize]();
                        __SLIC.pend(sw_interrupt);
                    },
                    _ => (#pac::__EXTERNAL_INTERRUPTS[hw_interrupt as usize]._handler)(),
                }
                #pac::PLIC::complete(hw_interrupt);
            }
        }
    }
}
