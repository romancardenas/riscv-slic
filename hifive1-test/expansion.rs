// Recursive expansion of codegen! macro
// ======================================

pub mod slic {
    use heapless::binary_heap::{BinaryHeap, Max};
    #[doc = r" Clears all interrupt flags to avoid interruptions."]
    #[inline(always)]
    pub unsafe fn clear_interrupts() {
        riscv::register::mstatus::clear_mie();
        riscv::register::mie::clear_mext();
        riscv::register::mie::clear_msoft();
        exti_clear();
        swi_clear();
        set_threshold(u8::MAX);
    }
    #[doc = r" Sets all the interrupt flags to allow external and software interrupts."]
    #[doc = r" It also sets the interrup threshold to 0 (i.e., accept all interrupts)."]
    #[inline(always)]
    pub unsafe fn set_interrupts() {
        set_threshold(0);
        riscv::register::mstatus::set_mie();
        riscv::register::mie::set_mext();
        riscv::register::mie::set_msoft();
    }
    #[doc = r" Sets the priority threshold of the external interrupt controller and the SLIC."]
    #[inline(always)]
    pub unsafe fn set_threshold(thresh: u8) {
        exti_set_threshold(thresh);
        __SLIC.set_threshold(thresh);
    }
    #[doc = r" Sets the interrupt priority of a given software interrupt"]
    #[doc = r" source in the external interrupt controller and the SLIC."]
    #[inline(always)]
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
    #[doc = r" Returns the current priority threshold of the SLIC."]
    #[inline(always)]
    pub unsafe fn get_threshold() -> u8 {
        __SLIC.get_threshold()
    }
    #[doc = r" Returns the interrupt priority of a given software interrupt source."]
    #[inline(always)]
    pub unsafe fn get_priority<I>(interrupt: I) -> u8
    where
        I: TryInto<Interrupt>,
        <I as TryInto<Interrupt>>::Error: core::fmt::Debug,
    {
        __SLIC.get_priority(interrupt.try_into().unwrap())
    }
    #[doc = r" Marks a software interrupt as pending."]
    #[inline(always)]
    pub unsafe fn pend<I>(interrupt: I)
    where
        I: TryInto<Interrupt>,
        <I as TryInto<Interrupt>>::Error: core::fmt::Debug,
    {
        __SLIC.pend(interrupt.try_into().unwrap());
    }
    #[doc = r" Runs a function with priority mask."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" If new priority is less than current priority, priority inversion may occur."]
    #[inline(always)]
    pub unsafe fn run<F: FnOnce()>(priority: u8, f: F) {
        let current = get_threshold();
        set_threshold(priority);
        f();
        set_threshold(current);
    }
    use riscv::peripheral::plic::PriorityNumber;
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
        plic.reset()
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
    impl TryFrom<u16> for Interrupt {
        type Error = u16;
        #[inline]
        fn try_from(value: u16) -> Result<Self, Self::Error> {
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
    pub static mut __SLIC: SLIC = SLIC::new();
    #[no_mangle]
    #[allow(non_snake_case)]
    pub unsafe fn MachineSoft() {
        swi_clear();
        while let Some((priority, interrupt)) = __SLIC.pop() {
            run(priority, || __SOFTWARE_INTERRUPTS[interrupt as usize]());
        }
    }
    #[doc = r" Software interrupt controller"]
    #[allow(clippy::upper_case_acronyms)]
    #[derive(Debug, Clone)]
    pub struct SLIC {
        #[doc = r" priority threshold. The controller only triggers software"]
        #[doc = r" interrupts if there is a pending interrupt with higher priority."]
        threshold: u8,
        #[doc = r" Array with the priorities assigned to each software interrupt source."]
        #[doc = r#" Priority 0 is reserved for "interrupt diabled"."#]
        priorities: [u8; 2usize],
        #[doc = r" Array to check if a software interrupt source is pending."]
        pending: [bool; 2usize],
        #[doc = r" Priority queue with pending interrupt sources."]
        queue: BinaryHeap<(u8, u16), Max, 2usize>,
    }
    impl SLIC {
        #[doc = r" Creates a new software interrupt controller"]
        #[inline]
        pub const fn new() -> Self {
            Self {
                threshold: 0,
                priorities: [0; 2usize],
                pending: [false; 2usize],
                queue: BinaryHeap::new(),
            }
        }
        #[inline(always)]
        fn get_threshold(&self) -> u8 {
            self.threshold
        }
        #[doc = r" Sets the priority threshold of the controller."]
        #[doc = r""]
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc = r" Changing the priority threshold may break priority-based critical sections."]
        #[inline(always)]
        unsafe fn set_threshold(&mut self, priority: u8) {
            self.threshold = priority;
        }
        #[doc = r" Returns the current priority of an interrupt source."]
        #[inline(always)]
        fn get_priority(&self, interrupt: Interrupt) -> u8 {
            self.priorities[interrupt as usize]
        }
        #[doc = r" Sets the priority of an interrupt source."]
        #[doc = r""]
        #[doc = r" # Note"]
        #[doc = r""]
        #[doc = r#" The 0 priority level is reserved for "never interrupt"."#]
        #[doc = r" Thus, when setting priority 0, it also clears the pending flag of the interrupt."]
        #[doc = r""]
        #[doc = r" Interrupts are queued according to their priority level when queued."]
        #[doc = r" Thus, if you change the priority of an interrupt while it is already queued,"]
        #[doc = r" the pending interrupt will execute with the previous priority."]
        #[doc = r""]
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc = r" Changing the priority level of an interrupt may break priority-based critical sections."]
        #[inline(always)]
        unsafe fn set_priority(&mut self, interrupt: Interrupt, priority: u8) {
            self.priorities[interrupt as usize] = priority;
        }
        #[doc = r" Checks if a given interrupt is pending."]
        #[inline(always)]
        fn is_pending(&mut self, interrupt: Interrupt) -> bool {
            self.pending[interrupt as usize]
        }
        #[doc = r" Returns `true` if the next queued interrupt can be triggered."]
        #[inline(always)]
        fn is_ready(&self) -> bool {
            match self.queue.peek() {
                Some(&(p, _)) => p > self.threshold,
                None => false,
            }
        }
        #[doc = r" Sets an interrupt source as pending."]
        #[doc = r""]
        #[doc = r" # Notes"]
        #[doc = r""]
        #[doc = r" If interrupt priority is 0 or already pending, this request is silently ignored."]
        #[inline(always)]
        fn pend(&mut self, interrupt: Interrupt) {
            let i = interrupt as usize;
            if self.priorities[i] == 0 || self.pending[i] {
                return;
            }
            self.pending[i] = true;
            unsafe {
                self.queue
                    .push_unchecked((self.priorities[i], interrupt as _))
            };
            if self.is_ready() {
                unsafe { swi_set() };
            }
        }
        #[doc = r" Executes all the pending tasks with high enough priority."]
        #[inline]
        fn pop(&mut self) -> Option<(u8, u16)> {
            match self.is_ready() {
                true => {
                    let (priority, interrupt) = unsafe { self.queue.pop_unchecked() };
                    self.pending[interrupt as usize] = false;
                    Some((priority, interrupt))
                }
                false => None,
            }
        }
    }
}
