use proc_macro2::TokenStream;
use quote::quote;

pub fn api_mod() -> TokenStream {
    quote!(
        /// Enables the software interrupt controller and triggers a software interrupt if ready.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_enable() {
            if __riscv_slic_cs(|slic| {
                slic.enable();
                slic.is_ready()
            }) {
                __riscv_slic_swi_pend();
            }
        }

        /// Disables the software interrupt controller and clears any pending software interrupt.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_disable() {
            __riscv_slic_cs(|slic| slic.disable());
        }

        /// Returns the current priority threshold of the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_get_threshold() -> u8 {
            __riscv_slic_cs(|slic| slic.get_threshold())
        }

        /// Sets the priority threshold of the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        ///
        /// Setting the priority threshold to a value lower than the current threshold
        /// may lead to priority inversion. If you want to make sure that the threshold
        /// is only raised, use the [`__riscv_slic_raise_threshold`] function instead.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_set_threshold(thresh: u8) {
            if __riscv_slic_cs(|slic| {
                slic.set_threshold(thresh);
                slic.is_ready()
            }) {
                __riscv_slic_swi_pend();
            }
        }

        /// Raises the priority threshold of the SLIC only if the new threshold is higher than the current one.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        ///
        /// This function is thought to be used as a way to temporarily raise the priority threshold.
        /// You must return the previous threshold to the SLIC after you are done.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_raise_threshold(priority: u8) -> Result<u8, u8> {
            let (res, is_ready) =
                __riscv_slic_cs(|slic| (slic.raise_threshold(priority), slic.is_ready()));
            // trigger a software interrupt if the SLIC is still ready at this point
            if is_ready {
                __riscv_slic_swi_pend();
            }
            res
        }

        /// Returns the interrupt priority of a given software interrupt source.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_get_priority(interrupt: u16) -> u8 {
            __riscv_slic_cs(|slic| slic.get_priority(interrupt))
        }

        /// Sets the interrupt priority of a given software interrupt source in the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_set_priority(interrupt: u16, priority: u8) {
            __riscv_slic_cs(|slic| slic.set_priority(interrupt, priority));
        }

        /// Marks a software interrupt as pending.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_pend(interrupt: u16) {
            if __riscv_slic_cs(|slic| {
                slic.pend(interrupt);
                slic.is_ready()
            }) {
                __riscv_slic_swi_pend();
            }
        }

        /// Polls the SLIC for a pending software interrupt and pops it from the pending queue.
        /// It also raises the threshold to the priority of the popped interrupt.
        ///
        /// This function returns a tuple (previous_threshold, interrupt_number).
        ///
        /// - `previous_threshold` is the previous threshold of the SLIC before the pop.
        /// - `interrupt_number` is the number of the popped interrupt.
        ///
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        ///
        /// Do not forget to restore the previous threshold after you are done with the interrupt.
        #[inline]
        #[unsafe(no_mangle)]
        unsafe fn __riscv_slic_pop() -> Option<(u8, u16)> {
            __riscv_slic_cs(|slic| {
                slic.pop().and_then(|(priority, interrupt)| {
                    let previous = unsafe { slic.raise_threshold(priority).unwrap() }; // must be Ok if pop returned Some!
                    Some((previous, interrupt))
                })
            })
        }
    )
}
