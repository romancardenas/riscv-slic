#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};
use riscv::register::{mie, mstatus};
use riscv_rt::entry;
use riscv_slic;

use hifive1::hal::e310x;

// generate SLIC code for this example, only adding a HW interrupt for
// GPIO4
// riscv_slic::codegen!(e310x, [GPIO4], []);

// Recursive expansion of codegen! macro
// ======================================

pub mod plic {
    use super::slic;
    impl TryFrom<e310x::Interrupt> for slic::Interrupt {
        type Error = e310x::Interrupt;
        fn try_from(value: e310x::Interrupt) -> Result<Self, Self::Error> {
            match value {
                e310x::Interrupt::GPIO4 => Ok(slic::Interrupt::GPIO4),
                _ => Err(value),
            }
        }
    }
    #[no_mangle]
    pub unsafe extern "C" fn MachineExternal() {
        if let Some(hw_interrupt) = e310x::PLIC::claim() {
            let sw_interrupt: Result<super::slic::Interrupt, e310x::Interrupt> =
                hw_interrupt.try_into();
            match sw_interrupt {
                Ok(sw_interrupt) => slic::__slic_pend(sw_interrupt as u16),
                _ => (e310x::__EXTERNAL_INTERRUPTS[hw_interrupt as usize - 1]._handler)(),
            }
            e310x::PLIC::complete(hw_interrupt);
        }
    }
}
pub mod slic {
    use heapless::binary_heap::{BinaryHeap, Max};
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
    pub unsafe fn __slic_get_threshold() -> u8 {
        __SLIC.get_threshold()
    }
    #[no_mangle]
    #[doc = r" (Visible externally) Set interrupt priority"]
    pub unsafe fn __slic_set_priority<I: TryInto<Interrupt>>(interrupt: I, priority: u8)
    where
        <I as TryInto<Interrupt>>::Error: core::fmt::Debug,
    {
        __SLIC.set_priority(interrupt.try_into().unwrap(), priority);
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
    #[doc = r" Enables machine software interrupts"]
    pub unsafe fn enable() {
        riscv::register::mie::set_msoft();
    }
    #[doc = r" Disables machine software interrupts"]
    pub unsafe fn disable() {
        riscv::register::mie::clear_msoft();
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
    #[derive(Clone, Copy, Eq, PartialEq)]
    #[repr(u16)]
    pub enum Interrupt {
        GPIO4 = 0,
    }
    impl Interrupt {
        #[inline]
        pub fn try_from(value: u16) -> Result<Self, u16> {
            match value {
                0 => Ok(Self::GPIO4),
                _ => Err(value),
            }
        }
    }
    extern "C" {
        fn GPIO4();

    }
    #[no_mangle]
    pub static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); 1usize] = [GPIO4];
    static mut __SLIC: SLIC = SLIC::new();
    #[no_mangle]
    pub unsafe fn MachineSoft() {
        clear_interrupt();
        __SLIC.pop(&__SOFTWARE_INTERRUPTS);
    }
}

// create a handler for GPIO4
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn GPIO4() {
    sprintln!("Hooray! We reached GPIO4 interrupt.");
    sprintln!("We got here with a priority of: ");
    unsafe {
        let prio = slic::__slic_get_priority(e310x::Interrupt::GPIO4 as u16);
        sprintln!("{0}", prio);
    }
    slic::clear_interrupt();
}

#[entry]
fn main() -> ! {
    let dr = unsafe { DeviceResources::steal() };

    let p = dr.peripherals;
    let pins = dr.pins;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 64.mhz().into());

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );
    let pin_gpio4 = pin!(pins, dig12);
    pin_gpio4.into_pull_up_input();

    sprintln!("Setting up the SLIC...");
    unsafe {
        slic::enable();
        mstatus::set_mie();
        sprintln!("Some threshold tests...");
        // slic::__slic_set_threshold(5);
        let thresh = slic::__slic_get_threshold();
        sprintln!("Current threshold: {0:?}", thresh);
        sprintln!("Setting some threshold...");
        slic::__slic_set_threshold(5);
        let thresh = slic::__slic_get_threshold();
        sprintln!("New threshold: {0:?}", thresh);
        sprintln!("Setting up interrupt for GPIO4");
        slic::__slic_set_priority(e310x::Interrupt::GPIO4, 1);
        sprintln!("Ready, now interrupt pin 12!");
        // Now wait for the interrupt to come...
    }
    // Disable watchdog
    let wdg = p.WDOG;
    wdg.wdogcfg.modify(|_, w| w.enalways().clear_bit());
    loop {}
}
