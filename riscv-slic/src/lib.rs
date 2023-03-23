#![no_std]

use heapless::binary_heap::{BinaryHeap, Max};

/// Re-export of codegen macro.
pub use riscv_slic_macros::codegen;

/// Software interrupt controller
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub struct SLIC<const N_INTERRUPTS: usize> {
    /// priority threshold. The controller only triggers software
    /// interrupts if there is a pending interrupt with higher priority.
    threshold: u8,
    /// Array with the priorities assigned to each software interrupt source.
    /// Priority 0 is reserved for "interrupt diabled".
    priorities: [u8; N_INTERRUPTS],
    /// Array to check if a software interrupt source is pending.
    pending: [bool; N_INTERRUPTS],
    /// Priority queue with pending interrupt sources.
    queue: BinaryHeap<(u8, u16), Max, N_INTERRUPTS>,
}

impl<const N_INTERRUPTS: usize> SLIC<N_INTERRUPTS> {
    /// Creates a new software interrupt controller
    #[inline]
    pub const fn new() -> Self {
        Self {
            threshold: 0,
            priorities: [0; N_INTERRUPTS],
            pending: [false; N_INTERRUPTS],
            queue: BinaryHeap::new(),
        }
    }

    //// Returns current priority threshold.
    #[inline(always)]
    pub fn get_threshold(&self) -> u8 {
        self.threshold
    }

    /// Sets the priority threshold of the controller.
    ///
    /// # Safety
    ///
    /// Changing the priority threshold may break priority-based critical sections.
    #[inline(always)]
    pub unsafe fn set_threshold(&mut self, priority: u8) {
        self.threshold = priority;
    }

    /// Returns the current priority of an interrupt source.
    #[inline(always)]
    pub fn get_priority(&self, interrupt: u16) -> u8 {
        self.priorities[interrupt as usize]
    }

    /// Sets the priority of an interrupt source.
    ///
    /// # Note
    ///
    /// The 0 priority level is reserved for "never interrupt".
    /// Thus, when setting priority 0, it also clears the pending flag of the interrupt.
    ///
    /// Interrupts are queued according to their priority level when queued.
    /// Thus, if you change the priority of an interrupt while it is already queued,
    /// the pending interrupt will execute with the previous priority.
    ///
    /// # Safety
    ///
    /// Changing the priority level of an interrupt may break priority-based critical sections.
    #[inline(always)]
    pub unsafe fn set_priority(&mut self, interrupt: u16, priority: u8) {
        self.priorities[interrupt as usize] = priority;
    }

    /// Checks if a given interrupt is pending.
    #[inline(always)]
    pub fn is_pending(&mut self, interrupt: u16) -> bool {
        self.pending[interrupt as usize]
    }

    /// Sets an interrupt source as pending.
    ///
    /// # Notes
    ///
    /// If interrupt priority is 0 or already pending, this request is silently ignored.
    #[inline(always)]
    pub fn pend(&mut self, interrupt: u16) {
        let i = interrupt as usize;
        if self.priorities[i] == 0 || self.pending[i] {
            return;
        }
        self.pending[i] = true;
        // SAFETY: we do not allow the same task to be pending more than once
        unsafe { self.queue.push_unchecked((self.priorities[i], interrupt)) };
        // Trigger a software interrupt when there is an interrupt awaiting
        if self.is_ready() {
            unsafe { self::export::set_interrupt() };
        }
    }

    /// Returns `true` if the next queued interrupt can be triggered.
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        match self.queue.peek() {
            Some(&(p, _)) => p > self.threshold,
            None => false,
        }
    }

    /// Executes all the pending tasks with high enough priority.
    ///
    /// # Safety
    ///
    /// This method is intended to be used only by the `MachineSoftware` interrupt handler.
    #[inline]
    pub unsafe fn pop(&mut self, handlers: &[unsafe extern "C" fn(); N_INTERRUPTS]) {
        export::clear_interrupt(); // We clear it at the beginning to allow nested interrupts
        while self.is_ready() {
            // SAFETY: we know there is at least one valid interrupt queued.
            let (priority, interrupt) = unsafe { self.queue.pop_unchecked() };
            self.run(priority, || unsafe { handlers[interrupt as usize]() });
            self.pending[interrupt as usize] = false; //task finishes only after running the handler
        }
    }

    /// Runs a function with priority mask.
    ///
    /// # Safety
    ///
    /// This method is intended to be used only by the `PLIC::pop` method.
    #[inline(always)]
    unsafe fn run<F: FnOnce()>(&mut self, priority: u8, f: F) {
        let current = self.get_threshold();
        self.set_threshold(priority);
        f();
        self.set_threshold(current);
    }
}

/// Enables machine software interrupts
pub unsafe fn enable() {
    riscv::register::mie::set_msoft();
}

/// Disables machine software interrupts
pub unsafe fn disable() {
    riscv::register::mie::clear_msoft();
}

pub mod export {
    #[cfg(feature = "e310x")]
    pub mod e310x {
        /// Triggers a machine software interrupt via the CLINT peripheral
        pub unsafe fn set_interrupt() {
            let clint = e310x::Peripherals::steal().CLINT;
            clint.msip.write(|w| w.bits(0x01));
        }

        /// Clears the Machine Software Interrupt Pending bit via the CLINT peripheral
        pub unsafe fn clear_interrupt() {
            let clint = e310x::Peripherals::steal().CLINT;
            clint.msip.write(|w| w.bits(0x00));
        }
    }

    #[cfg(feature = "e310x")]
    pub use self::e310x::*;
}
