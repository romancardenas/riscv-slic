use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Helper function for generating the interrupt enums. It assigns a number to each source.
fn interrupts_enum(input: &[Ident]) -> Vec<TokenStream> {
    input
        .iter()
        .enumerate()
        .map(|(i, interrupt)| format!("{interrupt} = {i}").parse().unwrap())
        .collect()
}

/// Helper function for the try_into method of interrupts. It retrieves the number of each source.
fn interrupts_into(input: &[Ident]) -> Vec<TokenStream> {
    input
        .iter()
        .enumerate()
        .map(|(i, interrupt)| format!("{i} => Ok(Self::{interrupt}),").parse().unwrap())
        .collect()
}

pub fn slic_mod(pac: &Ident, sw_handlers: &[Ident]) -> TokenStream {
    let n_interrupts = sw_handlers.len();
    let enums = interrupts_enum(&sw_handlers);
    let matches = interrupts_into(&sw_handlers);

    quote!(
       pub mod slic {
        use heapless::binary_heap::{BinaryHeap, Max};

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

        #[no_mangle]
        /// (Visible externally) Mark an interrupt as pending
        pub unsafe fn __slic_pend(interrupt: u16) {
            __SLIC.pend(self::Interrupt::try_from(interrupt).unwrap());
        }

        #[no_mangle]
        /// (Visible externally) Set the SLIC threshold
        pub unsafe fn __slic_set_threshold(thresh: u8) {
            __SLIC.set_threshold(thresh);
        }

        #[no_mangle]
        /// (Visible externally) Get SLIC threshold
        pub unsafe fn __slic_get_threshold() -> u8 {
            __SLIC.get_threshold()
        }

        #[no_mangle]
        /// (Visible externally) Set interrupt priority
        pub unsafe fn __slic_set_priority(interrupt: e310x::Interrupt, priority: u8) {
            __SLIC.set_priority(interrupt.try_into().unwrap(), priority);
        }

        #[no_mangle]
        /// (Visible externally) Get interrupt priority
        pub unsafe fn __slic_get_priority(interrupt: u16) -> u8 {
            __SLIC.get_priority(self::Interrupt::try_from(interrupt).unwrap())
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
                    unsafe { set_interrupt() };
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
            unsafe fn pop(&mut self, handlers: &[unsafe extern "C" fn(); #n_interrupts]) {
                clear_interrupt(); // We clear it at the beginning to allow nested interrupts
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

        /// Triggers a machine software interrupt via the CLINT peripheral
        pub unsafe fn set_interrupt() {
            let clint = #pac::Peripherals::steal().CLINT;
            clint.msip.write(|w| w.bits(0x01));
        }

        /// Clears the Machine Software Interrupt Pending bit via the CLINT peripheral
        pub unsafe fn clear_interrupt() {
            let clint = #pac::Peripherals::steal().CLINT;
            clint.msip.write(|w| w.bits(0x00));
        }

        #[derive(Clone, Copy, Eq, PartialEq)]
        #[repr(u16)]
        pub enum Interrupt {
            #(#enums),*
        }

        impl Interrupt {
            #[inline]
            pub fn try_from(value: u16) -> Result<Self, u16> {
                match value {
                    #(#matches)*
                    _ => Err(value),
                }
            }
            
        }

        extern "C" {
            #(fn #sw_handlers ();)*
        }

        #[no_mangle]
        pub static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); #n_interrupts] = [
            #(#sw_handlers),*
        ];

        static mut __SLIC: SLIC = SLIC::new();

        #[no_mangle]
        pub unsafe fn MachineSoft() {
            clear_interrupt();
            __SLIC.pop(&__SOFTWARE_INTERRUPTS);
        }
    }
    )
}
