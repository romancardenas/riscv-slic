use proc_macro2::TokenStream;
use quote::quote;

/// Creates the SLIC module with the proper interrupt sources.
pub fn api_mod() -> TokenStream {
    quote!(
        /// Clears all interrupt flags to avoid interruptions.
        pub unsafe fn clear_interrupts() {
            riscv::register::mstatus::clear_mie();
            riscv::register::mie::clear_mext();
            riscv::register::mie::clear_msoft();
            exti_clear();
            swi_clear();
            set_threshold(u8::MAX);
        }

        /// Sets all the interrupt flags to allow external and software interrupts.
        /// It also sets the interrup threshold to 0 (i.e., accept all interrupts).
        pub unsafe fn set_interrupts() {
            set_threshold(0);
            riscv::register::mstatus::set_mie();
            riscv::register::mie::set_mext();
            riscv::register::mie::set_msoft();
        }

        /// Sets the priority threshold of the external interrupt controller and the SLIC.
        pub unsafe fn set_threshold(thresh: u8) {
            exti_set_threshold(thresh);
            __SLIC.set_threshold(thresh);
        }

        /// Sets the interrupt priority of a given software interrupt
        /// source in the external interrupt controller and the SLIC.
        pub unsafe fn set_priority<I>(interrupt: I, priority: u8)
        where
            I: TryInto<Interrupt>,
            <I as TryInto<Interrupt>>::Error: core::fmt::Debug,
        {
            let swi: Interrupt = interrupt.try_into().unwrap();
            __SLIC.set_priority(swi, priority);
            if let Ok(exti) = swi.try_into() {
                exti_set_priority(exti, priority);
            }
        }

        /// Returns the current priority threshold of the SLIC.
        pub unsafe fn get_threshold() -> u8 {
            __SLIC.get_threshold()
        }

        /// Returns the interrupt priority of a given software interrupt source.
        pub unsafe fn get_priority<I>(interrupt: I) -> u8
        where
            I: TryInto<Interrupt>,
            <I as TryInto<Interrupt>>::Error: core::fmt::Debug,
        {
            __SLIC.get_priority(interrupt.try_into().unwrap())
        }

        /// Marks a software interrupt as pending.
        pub unsafe fn pend<I>(interrupt: I)
        where
            I: TryInto<Interrupt>,
            <I as TryInto<Interrupt>>::Error: core::fmt::Debug,
        {
            __SLIC.pend(interrupt.try_into().unwrap());
        }
    )
}
