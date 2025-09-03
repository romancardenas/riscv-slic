use crate::{
    export::export_swi_handler_attribute,
    input::{SwiAttr, SwiItem},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

fn interrupts_impl(slic: &Path, input: &SwiItem) -> TokenStream {
    let swi_name = &input.name;
    let sources = &input.sources;
    let n_interrupts = sources.len();

    let (mut from, mut to) = (Vec::new(), Vec::new());
    for (i, interrupt) in sources
        .iter()
        .enumerate()
        .map(|(i, interrupt)| (i as u16, interrupt))
    {
        to.push(quote! {
            #swi_name::#interrupt => #i,
        });
        from.push(quote! {
            #i => Ok(#swi_name::#interrupt),
        });
    }
    quote! {
        unsafe impl #slic::InterruptNumber for #swi_name {
            const MAX_INTERRUPT_NUMBER: u16 = #n_interrupts as u16 - 1;

            #[inline(always)]
            fn number(self) -> u16 {
                match self {
                    #(#to)*
                }
            }

            #[inline(always)]
            fn from_number(value: u16) -> Result<Self, u16> {
                match value {
                    #(#from)*
                    _ => Err(value),
                }
            }
        }
    }
}

/// Creates the SLIC module with the proper interrupt sources.
pub fn swi_mod(attr: &SwiAttr, item: &SwiItem) -> TokenStream {
    let mut res = Vec::new();

    let slic = &attr.slic;
    let swi_handlers = &item.sources;
    let n_interrupts = swi_handlers.len();
    let swi_handler_attribute = export_swi_handler_attribute(&attr.pac, &attr.backend);

    if n_interrupts > 0 {
        let swi_impl = interrupts_impl(slic, item);
        res.push(quote!(
            #swi_impl

            unsafe extern "C" {
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
        static __SLIC: #slic::slic::MutexSLIC<#n_interrupts> = #slic::slic::new_slic();

        /// Type alias for the SLIC instance that hides the number of interrupts.
        type SLIC = #slic::slic::SLIC<#n_interrupts>;

        /// Utility function to operate on the SLIC under a critical section.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        unsafe fn __riscv_slic_cs<F, R>(f: F) -> R
        where
            F: FnOnce(&mut SLIC) -> R,
        {
            #slic::critical_section::with(|cs| {
                let mut slic = __SLIC.borrow_ref_mut(cs);
                f(&mut slic)
            })
        }

        /// Software interrupt handler to be used with the SLIC.
        #swi_handler_attribute
        unsafe fn riscv_slic_swi_handler() {
            __riscv_slic_swi_unpend();
            // We nest the handler to let other interrupts trigger
            #slic::nested(|| {
                if let Some((prev, int)) = __riscv_slic_pop() {
                    __SOFTWARE_INTERRUPTS[int as usize]();
                    // SAFETY: we restore the previous threshold after the function is done
                    __riscv_slic_set_threshold(prev);
                }
             });
        }
    ));
    quote!(#(#res)*)
}
