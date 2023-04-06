use proc_macro2::TokenStream;
use quote::quote;

/// Creates the SLIC module with the proper interrupt sources.
pub fn slic_mod(n_interrupts: usize) -> TokenStream {
    quote!(
        /// Software interrupt controller
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Debug, Clone)]
        pub struct SLIC {
            /// priority threshold. The controller only triggers software
            /// interrupts if there is a pending interrupt with higher priority.
            threshold: u8,
            /// Array with the priorities assigned to each software interrupt source.
            /// Priority 0 is reserved for "interrupt diabled".
            priorities: [u8; #n_interrupts],
            /// Array to check if a software interrupt source is pending.
            pending: [bool; #n_interrupts],
            /// Priority queue with pending interrupt sources.
            queue: BinaryHeap<(u8, u16), Max, #n_interrupts>,
        }

        impl SLIC {
            /// Creates a new software interrupt controller
            #[inline]
            pub const fn new() -> Self {
                Self {
                    threshold: 0,
                    priorities: [0; #n_interrupts],
                    pending: [false; #n_interrupts],
                    queue: BinaryHeap::new(),
                }
            }

            //// Returns current priority threshold.
            #[inline(always)]
            fn get_threshold(&self) -> u8 {
                self.threshold
            }

            /// Sets the priority threshold of the controller.
            ///
            /// # Safety
            ///
            /// Changing the priority threshold may break priority-based critical sections.
            #[inline(always)]
            unsafe fn set_threshold(&mut self, priority: u8) {
                self.threshold = priority;
            }

            /// Returns the current priority of an interrupt source.
            #[inline(always)]
            fn get_priority(&self, interrupt: Interrupt) -> u8 {
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
            unsafe fn set_priority(&mut self, interrupt: Interrupt, priority: u8) {
                self.priorities[interrupt as usize] = priority;
            }

            /// Checks if a given interrupt is pending.
            #[inline(always)]
            fn is_pending(&mut self, interrupt: Interrupt) -> bool {
                self.pending[interrupt as usize]
            }

            /// Sets an interrupt source as pending.
            ///
            /// # Notes
            ///
            /// If interrupt priority is 0 or already pending, this request is silently ignored.
            #[inline(always)]
            fn pend(&mut self, interrupt: Interrupt) {
                let i = interrupt as usize;
                if self.priorities[i] == 0 || self.pending[i] {
                    return;
                }
                self.pending[i] = true;
                // SAFETY: we do not allow the same task to be pending more than once
                unsafe { self.queue.push_unchecked((self.priorities[i], interrupt as _)) };
                // Trigger a software interrupt when there is an interrupt awaiting
                if self.is_ready() {
                    unsafe { swi_set() };
                }
            }

            /// Returns `true` if the next queued interrupt can be triggered.
            #[inline(always)]
            fn is_ready(&self) -> bool {
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
            unsafe fn pop(&mut self, swi_handlers: &[unsafe extern "C" fn(); #n_interrupts]) {
                while self.is_ready() {
                    // SAFETY: we know there is at least one valid interrupt queued.
                    let (priority, interrupt) = self.queue.pop_unchecked();
                    self.run(priority, || swi_handlers[interrupt as usize]());
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
    )
}
