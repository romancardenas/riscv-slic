#![no_std]
#![no_main]
extern crate riscv_slic;

// Recursive expansion of codegen! macro
// ======================================

pub mod slic {
    use super::riscv_slic::*;
    use riscv_slic::InterruptNumber;
    #[doc = r" Returns the current priority threshold of the SLIC."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" This function is only for `riscv-slic` internal use. Do not call it directly."]
    #[inline]
    #[no_mangle]
    pub unsafe fn __riscv_slic_get_threshold() -> u8 {
        critical_section::with(|cs| __SLIC.borrow_ref(cs).get_threshold())
    }
    #[doc = r" Sets the priority threshold of the SLIC."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" This function is only for `riscv-slic` internal use. Do not call it directly."]
    #[inline]
    #[no_mangle]
    pub unsafe fn __riscv_slic_set_threshold(thresh: u8) {
        critical_section::with(|cs| {
            let mut slic = __SLIC.borrow_ref_mut(cs);
            slic.set_threshold(thresh);
            if slic.is_ready() {
                __riscv_slic_swi_pend();
            }
        });
    }
    #[doc = r" Returns the interrupt priority of a given software interrupt source."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" This function is only for `riscv-slic` internal use. Do not call it directly."]
    #[inline]
    #[no_mangle]
    pub unsafe fn __riscv_slic_get_priority(interrupt: u16) -> u8 {
        critical_section::with(|cs| __SLIC.borrow_ref(cs).get_priority(interrupt))
    }
    #[doc = r" Sets the interrupt priority of a given software interrupt source in the SLIC."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" This function is only for `riscv-slic` internal use. Do not call it directly."]
    #[inline]
    #[no_mangle]
    pub unsafe fn __riscv_slic_set_priority(interrupt: u16, priority: u8) {
        critical_section::with(|cs| __SLIC.borrow_ref_mut(cs).set_priority(interrupt, priority));
    }
    #[doc = r" Marks a software interrupt as pending."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" This function is only for `riscv-slic` internal use. Do not call it directly."]
    #[inline]
    #[no_mangle]
    pub unsafe fn __riscv_slic_pend(interrupt: u16) {
        critical_section::with(|cs| {
            let mut slic = __SLIC.borrow_ref_mut(cs);
            slic.pend(interrupt);
            if slic.is_ready() {
                __riscv_slic_swi_pend();
            }
        });
    }
    #[doc = r" Polls the SLIC for pending software interrupts and runs them."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" This function is only for `riscv-slic` internal use. Do not call it directly."]
    #[inline]
    #[no_mangle]
    pub unsafe fn __riscv_slic_run() {
        loop {
            let (priority, interrupt) = critical_section::with(|cs| {
                let mut slic = __SLIC.borrow_ref_mut(cs);
                match slic.pop() {
                    Some((priority, interrupt)) => {
                        slic.set_threshold(priority);
                        Some((priority, interrupt))
                    }
                    None => None,
                }
            });
            if let Some((priority, interrupt)) = (priority, interrupt) {
                riscv_slic::run(priority, || __SOFTWARE_INTERRUPTS[interrupt as usize]());
            }
        }
    }
    #[doc = r" Triggers a machine software interrupt via the CLINT peripheral."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" This function is only for `riscv-slic` internal use. Do not call it directly."]
    #[inline]
    #[no_mangle]
    pub unsafe fn __riscv_slic_swi_pend() {
        let msip = e310x::CLINT::mswi().msip(e310x::HartId::HART0);
        msip.pend();
    }
    #[doc = r" Clears the Machine Software Interrupt Pending bit via the CLINT peripheral."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" This function is only for `riscv-slic` internal use. Do not call it directly."]
    #[inline]
    #[no_mangle]
    pub unsafe fn __riscv_slic_swi_unpend() {
        let msip = e310x::CLINT::mswi().msip(e310x::HartId::HART0);
        msip.unpend();
    }
    #[doc = r" Enumeration of software interrupts"]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(u16)]
    pub enum Interrupt {
        SoftLow = 0,
        SoftMedium = 1,
        SoftHigh = 2,
    }
    unsafe impl InterruptNumber for Interrupt {
        const MAX_INTERRUPT_NUMBER: u16 = 3usize as u16 - 1;
        #[inline]
        fn number(self) -> u16 {
            self as _
        }
        #[inline]
        fn from_number(value: u16) -> Result<Self, u16> {
            if value > Self::MAX_INTERRUPT_NUMBER {
                Err(value)
            } else {
                Ok(unsafe { core::mem::transmute(value) })
            }
        }
    }
    extern "C" {
        fn SoftLow();

        fn SoftMedium();

        fn SoftHigh();

    }
    #[doc = r" Array of software interrupt handlers in the order of the `Interrupt` enum."]
    static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); 3usize] =
        [SoftLow, SoftMedium, SoftHigh];
    #[doc = r" The static SLIC instance"]
    static mut __SLIC: MutexSLIC<3usize> = new_slic();
    #[doc = r" Software interrupt handler to be used with the SLIC."]
    #[no_mangle]
    #[allow(non_snake_case)]
    unsafe fn MachineSoft() {
        __riscv_slic_swi_unpend();
        riscv::interrupt::nested(|| unsafe { __riscv_slic_run() });
    }
}
