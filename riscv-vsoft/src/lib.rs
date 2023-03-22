#![no_std]

use heapless::binary_heap::{BinaryHeap, Max};

/// Software interrupt controller
#[derive(Debug, Clone)]
pub struct SoftInterruptCtrl<const N_INTERRUPTS: usize> {
    /// priority threshold. The controller only triggers software
    /// interrupts if there is a pending interrupt with higher priority.
    threshold: u8,
    /// Array with the priorities assigned to each software interrupt source.
    priorities: [u8; N_INTERRUPTS],
    /// Array to check if a software interrupt source is pending.
    pending: [bool; N_INTERRUPTS],
    /// Priority queue with pending interrupt sources.
    queue: BinaryHeap<(u8, u16), Max, N_INTERRUPTS>,
}

impl<const N_INTERRUPTS: usize> SoftInterruptCtrl<N_INTERRUPTS> {
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
    #[inline]
    pub fn get_threshold(&self) -> u8 {
        self.threshold
    }

    /// Sets the priority threshold of the controller.
    ///
    /// # Safety
    ///
    /// Changing the priority threshold may break priority-based critical sections.
    #[inline]
    pub unsafe fn set_threshold(&mut self, priority: u8) {
        self.threshold = priority;
    }

    /// Returns the current priority of an interrupt source.
    #[inline]
    pub fn get_priority(&self, interrupt: u16) -> u8 {
        self.priorities[interrupt as usize]
    }

    /// Sets the priority of an interrupt source.
    ///
    /// # Note
    ///
    /// The 0 priority level is reserved for "never interrupt".
    #[inline]
    pub unsafe fn set_priority(&mut self, interrupt: u16, priority: u8) {
        self.priorities[interrupt as usize] = priority;
    }

    /// Checks is a given interrupt is pending.
    #[inline]
    pub fn is_pending(&mut self, interrupt: u16) -> bool {
        self.pending[interrupt as usize]
    }

    /// Sets an interrupt source as pending.
    ///
    /// # Notes
    ///
    /// If interrupt priority is 0 (i.e., "never interrupt")
    /// or already pending, this request is silently ignored.
    #[inline]
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

    /// Unpends an interrupt source.
    #[inline]
    fn unpend(&mut self, interrupt: u16) {
        self.pending[interrupt as usize] = false;
    }

    // Returns `true` if the next queued interrupt can be triggered.
    pub fn is_ready(&self) -> bool {
        match self.queue.peek() {
            Some(&(p, _)) => p > self.threshold,
            None => false,
        }
    }

    /// If not masked, pops the next pending interrupt and executes the corresponding handler.
    #[inline]
    pub unsafe fn pop(&mut self, handlers: &[unsafe extern "C" fn(); N_INTERRUPTS]) {
        if self.is_ready() {
            let (_, interrupt) = self.queue.pop_unchecked();
            handlers[interrupt as usize]();
            self.unpend(interrupt);
        }
        // Only clear interrupt pending bit if we are not ready
        if !self.is_ready() {
            unsafe { self::export::clear_interrupt() };
        }
    }
}

pub mod common {
    /// Enables machine software interrupts
    pub unsafe fn enable() {
        riscv::register::mie::set_msoft();
    }

    /// Disables machine software interrupts
    pub unsafe fn disable() {
        riscv::register::mie::clear_msoft();
    }
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
