use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Helper function for the [`TryFrom`] trait from hardware interrupts to software interrupts.
fn exti_to_sw(pac: &Ident, input: &[Ident]) -> Vec<TokenStream> {
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

fn exti_to_clear(input: &[Ident]) -> Vec<TokenStream> {
    input
        .iter()
        .map(|interrupt| format!("Clear{}", interrupt.to_string(),).parse().unwrap())
        .collect()
}

/// Creates the SLIC module with the proper interrupt sources.
pub fn exti_mod(pac: &Ident, exti_handlers: &[Ident]) -> TokenStream {
    let n_exti_interrupts = exti_handlers.len();
    if n_exti_interrupts == 0 {
        return quote!();
    }
    let exti_matches = exti_to_sw(pac, exti_handlers);
    let exti_clear = exti_to_clear(exti_handlers);

    quote! {
        impl TryFrom<#pac::Interrupt> for Interrupt {
            type Error = #pac::Interrupt;
            fn try_from(value: #pac::Interrupt) -> Result<Self, Self::Error> {
                match value {
                    #(#exti_matches)*
                    _ => Err(value),
                }
            }
        }

        extern "C" {
            #(fn #exti_clear ();)*
        }

        #[no_mangle]
        pub static __CLEAR_EXTERNAL_INTERRUPTS: [unsafe extern "C" fn(); #n_exti_interrupts] = [
            #(#exti_clear),*
        ];

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe fn MachineExternal() {
            if let Some(exti) = unsafe { exti_claim() } {
                let swi: Result<Interrupt, #pac::Interrupt> = exti.try_into();
                match swi {
                    Ok(swi) => {
                        __CLEAR_EXTERNAL_INTERRUPTS[swi as usize]();
                        __SLIC.pend(swi);
                    },
                    _ => (#pac::__EXTERNAL_INTERRUPTS[exti as usize]._handler)(),
                }
                unsafe {exti_complete(exti) };
            }
        }
    }
}
