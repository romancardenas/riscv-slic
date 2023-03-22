use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Returns a vector of with the identities of all the software interrupt handlers.
fn handlers_vec(n_interrupts: u16) -> Vec<TokenStream2> {
    (0..n_interrupts)
        .map(|n| format!("Software{n}").parse().unwrap())
        .collect()
}

#[proc_macro]
pub fn codegen(input: TokenStream) -> TokenStream {
    let tree: syn::LitInt = syn::parse(input).unwrap();
    let n_interrupts: usize = tree.base10_digits().parse::<usize>().unwrap();

    let interrupts = handlers_vec(n_interrupts as _);
    quote! {
        extern "C" {
            #(fn #interrupts ();)*
        }
        pub static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); #n_interrupts] = [
            #(#interrupts),*
        ];

        type SoftCtrl = riscv_vsoft::SoftInterruptCtrl<#n_interrupts>;
        pub static mut __SOFTWARE_CONTROLLER: SoftCtrl = SoftCtrl::new();

        #[no_mangle]
        pub unsafe fn SoftwareExternal() {
            __SOFTWARE_CONTROLLER.pop(&__SOFTWARE_INTERRUPTS);
        }
    }
    .into()
}
