use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;

/// Helper function for generating the interrupt enums. It assigns a number to each source.
fn interrupts_enum(input: &[Ident]) -> Vec<TokenStream2> {
    input
        .iter()
        .enumerate()
        .map(|(i, interrupt)| format!("{interrupt} = {i}").parse().unwrap())
        .collect()
}

/// Helper function for the try_into method of interrupts. It retrieves the number of each source.
fn interrupts_into(input: &[Ident]) -> Vec<TokenStream2> {
    input
        .iter()
        .enumerate()
        .map(|(i, interrupt)| format!("{i} => Ok(Self::{interrupt}),").parse().unwrap())
        .collect()
}

#[proc_macro]
pub fn codegen(input: TokenStream) -> TokenStream {
    let input: TokenStream2 = input.into();
    let mut interrupts_ident: Vec<Ident> = Vec::new();

    // TODO: parse the first argument as a device crate path, and the second argument as an array of dispatchers
    // ex. codegen!(e310x, [GPIO0, UART0, SWI0])
    let mut input_iterator = input.into_iter();

    // Get the device PAC
    let pac = match input_iterator.next() {
        Some(TokenTree::Ident(ident)) => Some(ident),
        _ => None,
    };
    let pac = pac.unwrap();
    // Get the separator
    let separator = match input_iterator.next() {
        Some(TokenTree::Punct(punct)) => Some(punct.as_char()),
        _ => None,
    };
    // Check for a comma separator
    assert_eq!(separator.unwrap(), ',');
    // Get the dispatchers array
    let dispatchers = match input_iterator.next() {
        Some(TokenTree::Group(array)) => Some(array),
        _ => None,
    };
    let dispatchers = dispatchers.unwrap();
    // Convert our group to a tokenstream
    let input_iterator = dispatchers.stream().into_iter();

    // INPUT TOKEN STREAM PARSING
    // Even tokens must be interrupt source identifiers, and odd tokens must be commas
    for (i, token) in input_iterator.enumerate() {
        if i % 2 == 0 {
            if let TokenTree::Ident(ident) = token {
                interrupts_ident.push(ident);
                continue;
            }
            return quote!(
                use invalid_input::input_must_be_interrupt_sources::separated_by_comma;
            )
            .into();
        } else {
            if let TokenTree::Punct(punct) = &token {
                if punct.as_char() == ',' {
                    continue;
                }
            }
            return quote!(
                use invalid_input::input_must_be_interrupt_sources::separated_by_comma;
            )
            .into();
        }
    }
    let n_interrupts: usize = interrupts_ident.len();
    // There must be at least one interrupt source
    if n_interrupts == 0 {
        return quote!(
            use invalid_input::you_must_define_at_least::one_interrupt_source::separated_by_comma;
        )
        .into();
    }

    let interrupts_enum = interrupts_enum(&interrupts_ident);
    let interrupts_into = interrupts_into(&interrupts_ident);

    quote! {

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
                    unsafe { set_interrupt() };
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
            pub unsafe fn pop(&mut self, handlers: &[unsafe extern "C" fn(); #n_interrupts]) {
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

        #[repr(u16)]
        pub enum Interrupt {
            #(#interrupts_enum),*
        }

        impl Interrupt {
            #[inline]
            pub fn try_from(value: u16) -> Result<Self, u16> {
                match value {
                    #(#interrupts_into)*
                    _ => Err(value),
                }
            }
        }

        extern "C" {
            #(fn #interrupts_ident ();)*
        }

        #[no_mangle]
        pub static __SOFTWARE_INTERRUPTS: [unsafe extern "C" fn(); #n_interrupts] = [
            #(#interrupts_ident),*
        ];

        #[no_mangle]
        pub static mut __SLIC: SLIC = SLIC::new();

        #[no_mangle]
        pub unsafe fn MachineSoft() {
            clear_interrupt();
            __SLIC.pop(&__SOFTWARE_INTERRUPTS);
        }
    }

        }
    .into()
}
