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
        #[no_mangle]
        pub unsafe fn __riscv_slic_enable() {
            critical_section::with(|cs| {
                if {
                    let mut slic = __SLIC.borrow_ref_mut(cs);
                    slic.enable()
                } {
                    // trigger a software interrupt if the SLIC is still ready at this point
                    __riscv_slic_swi_pend();
                }
            });
        }

        /// Disables the software interrupt controller and clears any pending software interrupt.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_disable() {
            critical_section::with(|cs| {
                __SLIC.borrow_ref_mut(cs).disable();
            });
        }

        /// Returns the current priority threshold of the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_get_threshold() -> u8 {
            critical_section::with(|cs| __SLIC.borrow_ref(cs).get_threshold())
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
        #[no_mangle]
        pub unsafe fn __riscv_slic_set_threshold(thresh: u8) {
            critical_section::with(|cs| {
                if {
                    let mut slic = __SLIC.borrow_ref_mut(cs);
                    slic.set_threshold(thresh);
                    slic.is_ready()
                } {
                    // trigger a software interrupt if the SLIC is still ready at this point
                    __riscv_slic_swi_pend();
                }
            });
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
        #[no_mangle]
        pub unsafe fn __riscv_slic_raise_threshold(priority: u8) -> Result<u8, ()> {
            critical_section::with(|cs| {
                let (res, is_ready) = {
                    let mut slic = __SLIC.borrow_ref_mut(cs);
                    let res = slic.raise_threshold(priority);
                    (res, slic.is_ready())
                };
                // trigger a software interrupt if the SLIC is still ready at this point
                if is_ready {
                    __riscv_slic_swi_pend();
                }
                res
            })
        }

        /// Returns the interrupt priority of a given software interrupt source.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_get_priority(interrupt: u16) -> u8 {
            critical_section::with(|cs| __SLIC.borrow_ref(cs).get_priority(interrupt))
        }

        /// Sets the interrupt priority of a given software interrupt source in the SLIC.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_set_priority(interrupt: u16, priority: u8) {
            critical_section::with(|cs| {
                __SLIC.borrow_ref_mut(cs).set_priority(interrupt, priority)
            });
        }

        /// Marks a software interrupt as pending.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_pend(interrupt: u16) {
            critical_section::with(|cs| {
                if {
                    let mut slic = __SLIC.borrow_ref_mut(cs);
                    slic.pend(interrupt);
                    slic.is_ready()
                } {
                    __riscv_slic_swi_pend();
                }
            });
        }

        /// Polls the SLIC for pending software interrupts and runs them.
        ///
        /// # Safety
        ///
        /// This function is only for `riscv-slic` internal use. Do not call it directly.
        #[inline]
        #[no_mangle]
        pub unsafe fn __riscv_slic_pop() {
            // We check if there are pending software interrupts and run them
            // Note that we must raise the threshold within the same critical section
            // to avoid corner cases where another interrupt is raised in between.
            if let Some((prev, int)) = critical_section::with(|cs| {
                let mut slic = __SLIC.borrow_ref_mut(cs);
                match slic.pop() {
                    Some((priority, interrupt)) => {
                        // SAFETY: we restore the previous threshold after the function is done
                        let previous = unsafe { slic.raise_threshold(priority).unwrap() }; // must be Ok if pop returned Some!
                        Some((previous, interrupt))
                    }
                    None => None,
                }
            }) {
                __SOFTWARE_INTERRUPTS[int as usize]();
                // SAFETY: we restore the previous threshold after the function is done
                unsafe { __riscv_slic_set_threshold(prev) };
            }
        }
    )
}
