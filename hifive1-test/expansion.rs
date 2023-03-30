// Recursive expansion of codegen! macro
// ======================================

pub mod slic {
    use heapless::binary_heap::{BinaryHeap, Max};
    #[doc = r" Enumeration of software interrupts"]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(u16)]
    pub enum Interrupt {
        RTC = 0,
    }
    impl TryFrom<u16> for Interrupt {
        type Error = u16;
        #[inline]
        fn try_from(value: u16) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Self::RTC),
                _ => Err(value),
            }
        }
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
    #[doc = r" Software interrupt controller"]
    #[allow(clippy::upper_case_acronyms)]
    #[derive(Debug, Clone)]
    pub struct SLIC {
        #[doc = r" priority threshold. The controller only triggers software"]
        #[doc = r" interrupts if there is a pending interrupt with higher priority."]
        threshold: u8,
        #[doc = r" Array with the priorities assigned to each software interrupt source."]
        #[doc = r#" Priority 0 is reserved for "interrupt diabled"."#]
        priorities: [u8; 1usize],
        #[doc = r" Array to check if a software interrupt source is pending."]
        pending: [bool; 1usize],
        #[doc = r" Priority queue with pending interrupt sources."]
        queue: BinaryHeap<(u8, u16), Max, 1usize>,
    }
    impl SLIC {
        #[doc = r" Creates a new software interrupt controller"]
        #[inline]
        pub const fn new() -> Self {
            Self {
                threshold: 0,
                priorities: [0; 1usize],
                pending: [false; 1usize],
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
                unsafe { set_interrupt() };
            }
        }
        #[doc = r" Returns `true` if the next queued interrupt can be triggered."]
        #[inline(always)]
        fn is_ready(&self) -> bool {
            match self.queue.peek() {
                Some(&(p, _)) => p > self.threshold,
                None => false,
            }
        }
        #[doc = r" Executes all the pending tasks with high enough priority."]
        #[doc = r""]
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc = r" This method is intended to be used only by the `MachineSoftware` interrupt handler."]
        #[inline]
        unsafe fn pop(&mut self, handlers: &[unsafe extern "C" fn(); 1usize]) {
            clear_interrupt();
            while self.is_ready() {
                let (priority, interrupt) = unsafe { self.queue.pop_unchecked() };
                self.run(priority, || unsafe { handlers[interrupt as usize]() });
                self.pending[interrupt as usize] = false;
            }
        }
        #[doc = r" Runs a function with priority mask."]
        #[doc = r""]
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc = r" This method is intended to be used only by the `PLIC::pop` method."]
        #[inline(always)]
        unsafe fn run<F: FnOnce()>(&mut self, priority: u8, f: F) {
            let current = self.get_threshold();
            self.set_threshold(priority);
            f();
            self.set_threshold(current);
        }
    }
    #[doc = r" Triggers a machine software interrupt via the CLINT peripheral"]
    pub unsafe fn set_interrupt() {
        let clint = e310x::Peripherals::steal().CLINT;
        clint.msip.write(|w| w.bits(0x01));
    }
    #[doc = r" Clears the Machine Software Interrupt Pending bit via the CLINT peripheral"]
    pub unsafe fn clear_interrupt() {
        let clint = e310x::Peripherals::steal().CLINT;
        clint.msip.write(|w| w.bits(0x00));
    }
    extern "C" {
        fn RTC();

    }
    #[no_mangle]
    pub static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); 1usize] = [RTC];
    pub static mut __SLIC: SLIC = SLIC::new();
    #[no_mangle]
    #[allow(non_snake_case)]
    pub unsafe fn MachineSoft() {
        clear_interrupt();
        __SLIC.pop(&__SOFTWARE_INTERRUPTS);
    }
    #[no_mangle]
    #[allow(non_snake_case)]
    pub unsafe fn MachineExternal() {
        if let Some(hw_interrupt) = e310x::PLIC::claim() {
            let sw_interrupt: Result<Interrupt, e310x::Interrupt> = hw_interrupt.try_into();
            match sw_interrupt {
                Ok(sw_interrupt) => __SLIC.pend(sw_interrupt),
                _ => (e310x::__EXTERNAL_INTERRUPTS[hw_interrupt as usize]._handler)(),
            }
            e310x::PLIC::complete(hw_interrupt);
        }
    }
    #[doc = r" (Visible externally) Set the SLIC threshold"]
    pub unsafe fn set_threshold(thresh: u8) {
        __SLIC.set_threshold(thresh);
    }
    #[doc = r" (Visible externally) Get SLIC threshold"]
    pub unsafe fn slic_get_threshold() -> u8 {
        __SLIC.get_threshold()
    }
    #[doc = r" (Visible externally) Mark an interrupt as pending"]
    pub unsafe fn pend<I>(interrupt: I)
    where
        I: TryInto<Interrupt>,
        <I as TryInto<Interrupt>>::Error: core::fmt::Debug,
    {
        __SLIC.pend(interrupt.try_into().unwrap());
    }
    #[doc = r" (Visible externally) Set interrupt priority"]
    pub unsafe fn set_priority<I>(interrupt: I, priority: u8)
    where
        I: TryInto<Interrupt>,
        <I as TryInto<Interrupt>>::Error: core::fmt::Debug,
    {
        __SLIC.set_priority(interrupt.try_into().unwrap(), priority);
    }
    #[doc = r" (Visible externally) Get interrupt priority"]
    pub unsafe fn get_priority<I>(interrupt: I) -> u8
    where
        I: TryInto<Interrupt>,
        <I as TryInto<Interrupt>>::Error: core::fmt::Debug,
    {
        __SLIC.get_priority(interrupt.try_into().unwrap())
    }
}
