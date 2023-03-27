#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod plic {
    use super::slic;
    impl TryFrom<e310x::Interrupt> for slic::Interrupt {
        type Error = e310x::Interrupt;
        fn try_from(value: e310x::Interrupt) -> Result<Self, Self::Error> {
            match value {
                e310x::Interrupt::GPIO0 => Ok(slic::Interrupt::GPIO0),
                e310x::Interrupt::GPIO1 => Ok(slic::Interrupt::GPIO1),
                e310x::Interrupt::UART0 => Ok(slic::Interrupt::UART0),
                _ => Err(value),
            }
        }
    }
    #[no_mangle]
    pub unsafe extern "C" fn MachineExternal() {
        if let Some(hw_interrupt) = e310x::PLIC::claim() {
                let sw_interrupt:
                        Result<super::slic::Interrupt, e310x::Interrupt> =
                    hw_interrupt.try_into();
                match sw_interrupt {
                    Ok(sw_interrupt) => slic::__slic_pend(sw_interrupt as u16),
                    _ =>
                        (e310x::__EXTERNAL_INTERRUPTS[hw_interrupt as usize -
                                            1]._handler)(),
                }
                e310x::PLIC::complete(hw_interrupt);
            }
    }
}
pub mod slic {
    use heapless::binary_heap::{BinaryHeap, Max};
    #[doc = r" Software interrupt controller"]
    #[allow(clippy :: upper_case_acronyms)]
    pub struct SLIC {
        #[doc = r" priority threshold. The controller only triggers software"]
        #[doc =
        r" interrupts if there is a pending interrupt with higher priority."]
        threshold: u8,
        #[doc =
        r" Array with the priorities assigned to each software interrupt source."]
        #[doc = r#" Priority 0 is reserved for "interrupt diabled"."#]
        priorities: [u8; 5usize],
        #[doc = r" Array to check if a software interrupt source is pending."]
        pending: [bool; 5usize],
        #[doc = r" Priority queue with pending interrupt sources."]
        queue: BinaryHeap<(u8, u16), Max, 5usize>,
    }
    #[automatically_derived]
    #[allow(clippy :: upper_case_acronyms)]
    impl ::core::fmt::Debug for SLIC {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(f, "SLIC",
                "threshold", &self.threshold, "priorities", &self.priorities,
                "pending", &self.pending, "queue", &&self.queue)
        }
    }
    #[automatically_derived]
    #[allow(clippy :: upper_case_acronyms)]
    impl ::core::clone::Clone for SLIC {
        #[inline]
        fn clone(&self) -> SLIC {
            SLIC {
                threshold: ::core::clone::Clone::clone(&self.threshold),
                priorities: ::core::clone::Clone::clone(&self.priorities),
                pending: ::core::clone::Clone::clone(&self.pending),
                queue: ::core::clone::Clone::clone(&self.queue),
            }
        }
    }
    #[no_mangle]
    #[doc = r" (Visible externally) Mark an interrupt as pending"]
    pub unsafe fn __slic_pend(interrupt: u16) {
        __SLIC.pend(self::Interrupt::try_from(interrupt).unwrap());
    }
    #[no_mangle]
    #[doc = r" (Visible externally) Set the SLIC threshold"]
    pub unsafe fn __slic_set_threshold(thresh: u8) {
        __SLIC.set_threshold(thresh);
    }
    #[no_mangle]
    #[doc = r" (Visible externally) Get SLIC threshold"]
    pub unsafe fn __slic_get_threshold() -> u8 { __SLIC.get_threshold() }
    #[no_mangle]
    #[doc = r" (Visible externally) Set interrupt priority"]
    pub unsafe fn __slic_set_priority(interrupt: u16, priority: u8) {
        __SLIC.set_priority(self::Interrupt::try_from(interrupt).unwrap(),
            priority);
    }
    #[no_mangle]
    #[doc = r" (Visible externally) Get interrupt priority"]
    pub unsafe fn __slic_get_priority(interrupt: u16) -> u8 {
        __SLIC.get_priority(self::Interrupt::try_from(interrupt).unwrap())
    }
    impl SLIC {
        #[doc = r" Creates a new software interrupt controller"]
        #[inline]
        pub const fn new() -> Self {
            Self {
                threshold: 0,
                priorities: [0; 5usize],
                pending: [false; 5usize],
                queue: BinaryHeap::new(),
            }
        }
        #[inline(always)]
        fn get_threshold(&self) -> u8 { self.threshold }
        #[doc = r" Sets the priority threshold of the controller."]
        #[doc = r""]
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc =
        r" Changing the priority threshold may break priority-based critical sections."]
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
        #[doc =
        r" Thus, when setting priority 0, it also clears the pending flag of the interrupt."]
        #[doc = r""]
        #[doc =
        r" Interrupts are queued according to their priority level when queued."]
        #[doc =
        r" Thus, if you change the priority of an interrupt while it is already queued,"]
        #[doc =
        r" the pending interrupt will execute with the previous priority."]
        #[doc = r""]
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc =
        r" Changing the priority level of an interrupt may break priority-based critical sections."]
        #[inline(always)]
        unsafe fn set_priority(&mut self, interrupt: Interrupt,
            priority: u8) {
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
        #[doc =
        r" If interrupt priority is 0 or already pending, this request is silently ignored."]
        #[inline(always)]
        fn pend(&mut self, interrupt: Interrupt) {
            let i = interrupt as usize;
            if self.priorities[i] == 0 || self.pending[i] { return; }
            self.pending[i] = true;
            unsafe {
                self.queue.push_unchecked((self.priorities[i],
                        interrupt as _))
            };
            if self.is_ready() { unsafe { set_interrupt() }; }
        }
        #[doc =
        r" Returns `true` if the next queued interrupt can be triggered."]
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
        #[doc =
        r" This method is intended to be used only by the `MachineSoftware` interrupt handler."]
        #[inline]
        unsafe fn pop(&mut self,
            handlers: &[unsafe extern "C" fn(); 5usize]) {
            clear_interrupt();
            while self.is_ready() {
                let (priority, interrupt) =
                    unsafe { self.queue.pop_unchecked() };
                self.run(priority,
                    || unsafe { handlers[interrupt as usize]() });
                self.pending[interrupt as usize] = false;
            }
        }
        #[doc = r" Runs a function with priority mask."]
        #[doc = r""]
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc =
        r" This method is intended to be used only by the `PLIC::pop` method."]
        #[inline(always)]
        unsafe fn run<F: FnOnce()>(&mut self, priority: u8, f: F) {
            let current = self.get_threshold();
            self.set_threshold(priority);
            f();
            self.set_threshold(current);
        }
    }
    #[doc = r" Enables machine software interrupts"]
    pub unsafe fn enable() { riscv::register::mie::set_msoft(); }
    #[doc = r" Disables machine software interrupts"]
    pub unsafe fn disable() { riscv::register::mie::clear_msoft(); }
    #[doc =
    r" Triggers a machine software interrupt via the CLINT peripheral"]
    pub unsafe fn set_interrupt() {
        let clint = e310x::Peripherals::steal().CLINT;
        clint.msip.write(|w| w.bits(0x01));
    }
    #[doc =
    r" Clears the Machine Software Interrupt Pending bit via the CLINT peripheral"]
    pub unsafe fn clear_interrupt() {
        let clint = e310x::Peripherals::steal().CLINT;
        clint.msip.write(|w| w.bits(0x00));
    }
    #[repr(u16)]
    pub enum Interrupt {
        Soft1 = 0,
        Soft3 = 1,
        GPIO0 = 2,
        GPIO1 = 3,
        UART0 = 4,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Interrupt {
        #[inline]
        fn clone(&self) -> Interrupt { *self }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Interrupt { }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Interrupt { }
    #[automatically_derived]
    impl ::core::cmp::Eq for Interrupt {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Interrupt { }
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Interrupt {
        #[inline]
        fn eq(&self, other: &Interrupt) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    impl Interrupt {
        #[inline]
        pub fn try_from(value: u16) -> Result<Self, u16> {
            match value {
                0 => Ok(Self::Soft1),
                1 => Ok(Self::Soft3),
                2 => Ok(Self::GPIO0),
                3 => Ok(Self::GPIO1),
                4 => Ok(Self::UART0),
                _ => Err(value),
            }
        }
    }
    extern "C" {
        fn Soft1();
        fn Soft3();
        fn GPIO0();
        fn GPIO1();
        fn UART0();
    }
    #[no_mangle]
    pub static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); 5usize] =
        [Soft1, Soft3, GPIO0, GPIO1, UART0];
    static mut __SLIC: SLIC = SLIC::new();
    #[no_mangle]
    pub unsafe fn MachineSoft() {
        clear_interrupt();
        __SLIC.pop(&__SOFTWARE_INTERRUPTS);
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn GPIO0() {}

#[no_mangle]
#[allow(non_snake_case)]
pub fn GPIO1() {}