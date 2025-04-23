use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

mod api;
mod export;
mod input;
mod swi;

#[proc_macro_attribute]
/// Generates all the necessary code for creating a SLIC struct and its associated functions.
/// This attribute must be applied to an enum that represents the desired software interrupts.
///
/// # Usage
///
/// ```ignore
/// #[swi([slic = riscv_slic,] pac = <pac::crate>[, backend = <backend-specific configuration>])]
/// #[derive(Debug, Clone, Copy)]
/// enum SoftwareInterrupt {
///     List,
///     Of,
///     Software,
///     Interrupts,
/// }
/// ```
///
/// # Item arguments
///
/// The item argument must be an enum that represents the software interrupts.
/// The enum variants should be named according to the software interrupts they represent.
/// The enum can be empty, but it is recommended to have at least one variant.
///
/// ## Notes
///
/// - The enum must implement the `Copy` trait.
/// - While the enum may include discriminants, the macro will ignore them. Instead, it will
///   assign the interrupt numbers based on the order of the variants.
///
/// # Attribute arguments
///
/// * `slic` - Path to the SLIC crate. This is optional and defaults to `riscv_slic`.
/// * `pac` - Path to the peripheral access crate for the target device. This is required.
/// * `backend` - The backend-specific configuration. Depending on the backend, this may be required or optional.
///
/// # Backend-specific configuration
///
/// ## `clint-backend`
///
/// * `hart_id` - The identifier of the HART whose software interrupt should be triggered.
///
/// ### Example
///
/// ```ignore
/// #[swi(pac = e310x, backend = { hart_id = H0 })]
/// #[derive(Debug, Clone, Copy)]
/// enum SoftwareInterrupt {
///     List,
///     Of,
///     Software,
///     Interrupts,
/// }
/// ```
///
/// ## `mecall-backend`
///
/// This backend does not require any configuration.
///
/// ### Example
///
/// ```ignore
/// #[swi(pac = e310x)]
/// #[derive(Debug, Clone, Copy)]
/// enum SoftwareInterrupt {
///     List,
///     Of,
///     Software,
///     Interrupts,
/// }
/// ```
pub fn swi(attr: TokenStream, item: TokenStream) -> TokenStream {
    let swi_attr = syn::parse_macro_input!(attr as input::SwiAttr);
    let swi_item = item.clone();
    let swi_item = syn::parse_macro_input!(swi_item as input::SwiItem);
    let item = syn::parse_macro_input!(item as DeriveInput);

    let pac = &swi_attr.pac;
    let swi_name = &swi_item.name;

    let swi_export = export::export_quote(&swi_attr);
    let swi_code = swi::swi_mod(&swi_attr, &swi_item);
    let api_code = api::api_mod();

    quote! {
        #item
        /// RISC-V SLIC private module. Do not use this module directly.
        mod _riscv_slic {
            use super::{#pac, #swi_name};

            #swi_export

            #swi_code

            #api_code
        }
    }
    .into()
}

#[proc_macro_attribute]
/// Attribute to declare a software interrupt handler.
///
/// The function must have the signature `[unsafe] fn() [-> !]`.
///
/// The argument of the macro must be a path to a variant of an enum that implements the
/// `riscv_slic::InterruptNumber` trait. This is the same enum used in the `#[swi]` macro.
///
/// # Example
///
/// ``` ignore,no_run
/// #[riscv_slic::interrupt(SoftwareInterrupt::List)]
/// fn hadler_for_list() -> ! {
///     loop{};
/// }
/// ```
pub fn interrupt(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as input::InterruptItem);
    let attr = parse_macro_input!(attr as input::InterruptAttr);

    let slic = &attr.slic;
    let int_path = attr.source;
    let int_ident = &int_path.segments.last().unwrap().ident;
    let export_name = format!("{:#}", int_ident);
    let f = item.0;

    quote!(
        // Compile-time check to ensure the trap path implements the trap trait
        const _: fn() = || {
            fn assert_impl<T: #slic::InterruptNumber>(_arg: T) {}
            assert_impl(#int_path);
        };

        #[unsafe(export_name = #export_name)]
        #f
    )
    .into()
}
