use proc_macro2::{Ident, TokenStream};
use quote::quote;

fn hw_to_sw(pac: &Ident, input: &[Ident]) -> Vec<TokenStream> {
    input
        .iter()
        .map(|interrupt| {
            format!(
                "{}::Interrupt::{} => Ok(slic::Interrupt::{}),",
                pac.to_string(),
                interrupt.to_string(),
                interrupt.to_string(),
            )
            .parse()
            .unwrap()
        })
        .collect()
}

pub fn plic_mod(pac: &Ident, bypassed_hw: &[Ident]) -> TokenStream {
    let matches = hw_to_sw(pac, bypassed_hw);
    quote!(
        pub mod plic {
            use super::slic;

            impl TryFrom<#pac::Interrupt> for slic::Interrupt {
                type Error = #pac::Interrupt;
                fn try_from(value: #pac::Interrupt) -> Result<Self, Self::Error> {
                    match value {
                        #(#matches)*
                        _ => Err(value),
                    }
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn MachineExternal() {
                if let Some(hw_interrupt) = #pac::PLIC::claim() {

                    match hw_interrupt.try_into() {
                        Ok(sw_interrupt) => slic::__SLIC.pend(sw_interrupt),
                        _ => {}, // _ => #pac::__EXTERNAL_INTERRUPTS[hw_interrupt as usize - 1], TODO esto falta en la
                    }

                    #pac::PLIC::complete(hw_interrupt);
                }
            }
        }
    )
}
