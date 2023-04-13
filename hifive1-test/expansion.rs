// Recursive expansion of codegen! macro
// ======================================

pub mod slic {
    use riscv_slic::swi::InterruptNumber;
    #[doc = r" Clears all interrupt flags to avoid interruptions of SLIC and HW controller."]
    #[inline(always)]
    #[no_mangle]
    pub unsafe fn __slic_clear() {
        exti_clear();
        swi_clear();
    }
    #[doc = r" Returns the current priority threshold of the SLIC."]
    #[inline(always)]
    #[no_mangle]
    pub unsafe fn __slic_get_threshold() -> u8 {
        __SLIC.get_threshold()
    }
    #[doc = r" Sets the priority threshold of the external interrupt controller and the SLIC."]
    #[inline(always)]
    #[no_mangle]
    pub unsafe fn __slic_set_threshold(thresh: u8) {
        exti_set_threshold(thresh);
        __SLIC.set_threshold(thresh);
    }
    #[doc = r" Returns the interrupt priority of a given software interrupt source."]
    #[inline(always)]
    #[no_mangle]
    pub unsafe fn __slic_get_priority(interrupt: u16) -> u8 {
        let interrupt: Interrupt = InterruptNumber::try_from(interrupt).unwrap();
        __SLIC.get_priority(interrupt)
    }
    #[doc = r" Sets the interrupt priority of a given software interrupt"]
    #[doc = r" source in the external interrupt controller and the SLIC."]
    #[inline(always)]
    #[no_mangle]
    pub unsafe fn __slic_set_priority(interrupt: u16, priority: u8) {
        let interrupt: Interrupt = InterruptNumber::try_from(interrupt).unwrap();
        __SLIC.set_priority(interrupt, priority);
        if let Ok(exti) = interrupt.try_into() {
            exti_set_priority(exti, priority);
        }
    }
    #[doc = r" Marks a software interrupt as pending."]
    #[inline(always)]
    #[no_mangle]
    pub unsafe fn __slic_pend(interrupt: u16) {
        let interrupt: Interrupt = InterruptNumber::try_from(interrupt).unwrap();
        __SLIC.pend(interrupt);
        if __SLIC.is_ready() {
            swi_set();
        }
    }
    use riscv_slic::exti::PriorityNumber;
    #[doc = r" Converts an `u8` to the corresponding priority level."]
    #[doc = r" If conversion fails, it returns the highest available priority level."]
    #[inline(always)]
    fn saturated_priority(mut priority: u8) -> e310x::Priority {
        if priority > e310x::Priority::MAX_PRIORITY_NUMBER {
            priority = e310x::Priority::MAX_PRIORITY_NUMBER;
        }
        e310x::Priority::try_from(priority).unwrap()
    }
    #[inline(always)]
    unsafe fn exti_clear() {
        let mut plic = e310x::Peripherals::steal().PLIC;
        plic.reset();
    }
    #[doc = r" Returns the next pending external interrupt according to the PLIC."]
    #[doc = r" If no external interrupts are pending, it returns `None`."]
    #[inline(always)]
    fn exti_claim() -> Option<e310x::Interrupt> {
        e310x::PLIC::claim()
    }
    #[doc = r" Notifies the PLIC that a pending external interrupt as complete."]
    #[doc = r" If the interrupt was not pending, it silently ignores it."]
    #[inline(always)]
    fn exti_complete(exti: e310x::Interrupt) {
        e310x::PLIC::complete(exti);
    }
    #[doc = r" Sets the PLIC threshold to the desired value. If threshold is higher than"]
    #[doc = r" the highest priority, it sets the threshold to the highest possible value."]
    #[inline(always)]
    unsafe fn exti_set_threshold(threshold: u8) {
        let mut plic = e310x::Peripherals::steal().PLIC;
        plic.set_threshold(saturated_priority(threshold));
    }
    #[doc = r" Enables the PLIC interrupt source and sets its priority to the desired value."]
    #[doc = r" If priority is higher than the highest priority, it sets it to the highest possible value."]
    #[inline(always)]
    unsafe fn exti_set_priority(interrupt: e310x::Interrupt, priority: u8) {
        let mut plic = e310x::Peripherals::steal().PLIC;
        plic.enable_interrupt(interrupt);
        plic.set_priority(interrupt, saturated_priority(priority));
    }
    impl TryFrom<e310x::Interrupt> for Interrupt {
        type Error = e310x::Interrupt;
        fn try_from(value: e310x::Interrupt) -> Result<Self, Self::Error> {
            match value {
                e310x::Interrupt::RTC => Ok(Interrupt::RTC),
                _ => Err(value),
            }
        }
    }
    impl TryFrom<Interrupt> for e310x::Interrupt {
        type Error = Interrupt;
        fn try_from(value: Interrupt) -> Result<Self, Self::Error> {
            match value {
                Interrupt::RTC => Ok(e310x::Interrupt::RTC),
                _ => Err(value),
            }
        }
    }
    extern "C" {
        fn ClearRTC();

    }
    #[no_mangle]
    pub static __CLEAR_EXTERNAL_INTERRUPTS: [unsafe extern "C" fn(); 1usize] = [ClearRTC];
    #[no_mangle]
    #[allow(non_snake_case)]
    pub unsafe fn MachineExternal() {
        if let Some(exti) = unsafe { exti_claim() } {
            let swi: Result<Interrupt, e310x::Interrupt> = exti.try_into();
            match swi {
                Ok(swi) => {
                    __CLEAR_EXTERNAL_INTERRUPTS[swi as usize]();
                    __SLIC.pend(swi);
                }
                _ => (e310x::__EXTERNAL_INTERRUPTS[exti as usize]._handler)(),
            }
            unsafe { exti_complete(exti) };
        }
    }
    #[doc = r" Triggers a machine software interrupt via the CLINT peripheral"]
    #[inline(always)]
    pub unsafe fn swi_set() {
        let clint = e310x::Peripherals::steal().CLINT;
        clint.msip.write(|w| w.bits(0x01));
    }
    #[doc = r" Clears the Machine Software Interrupt Pending bit via the CLINT peripheral"]
    #[inline(always)]
    pub unsafe fn swi_clear() {
        let clint = e310x::Peripherals::steal().CLINT;
        clint.msip.write(|w| w.bits(0x00));
    }
    #[doc = r" Enumeration of software interrupts"]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(u16)]
    pub enum Interrupt {
        RTC = 0,
        SoftLow = 1,
    }
    unsafe impl riscv_slic::swi::InterruptNumber for Interrupt {
        const MAX_INTERRUPT_NUMBER: u16 = 2usize as u16 - 1;
        fn number(self) -> u16 {
            self as _
        }
        fn try_from(value: u16) -> Result<Self, u16> {
            match value {
                0 => Ok(Self::RTC),
                1 => Ok(Self::SoftLow),
                _ => Err(value),
            }
        }
    }
    extern "C" {
        fn RTC();

        fn SoftLow();

    }
    #[no_mangle]
    pub static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); 2usize] = [RTC, SoftLow];
    pub static mut __SLIC: riscv_slic::SLIC<2usize> = riscv_slic::SLIC::new();
    #[no_mangle]
    #[allow(non_snake_case)]
    pub unsafe fn MachineSoft() {
        swi_clear();
        while let Some((priority, interrupt)) = __SLIC.pop() {
            riscv_slic::run(priority, || __SOFTWARE_INTERRUPTS[interrupt as usize]());
        }
    }
}
