use super::swi::InterruptNumber;
use atomic_polyfill::{AtomicBool, AtomicU8, Ordering};
use heapless::binary_heap::{BinaryHeap, Max};

/// Software interrupt controller
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub struct SLIC<const N: usize> {
    /// priority threshold. The controller only triggers software
    /// interrupts if there is a pending interrupt with higher priority.
    threshold: AtomicU8,
    /// Array with the priorities assigned to each software interrupt source.
    /// Priority 0 is reserved for "interrupt diabled".
    priorities: [u8; N],
    /// Array to check if a software interrupt source is pending.
    pending: [AtomicBool; N],
    /// Priority queue with pending interrupt sources.
    queue: BinaryHeap<(u8, u16), Max, N>,
}

// Hack to create an array of atomic booleans statically.
#[allow(clippy::declare_interior_mutable_const)]
const DEFAULT_ATOMIC: AtomicBool = AtomicBool::new(false);

impl<const N: usize> SLIC<N> {
    /// Creates a new software interrupt controller
    #[inline]
    pub const fn new() -> Self {
        Self {
            threshold: AtomicU8::new(0),
            priorities: [0; N],
            pending: [DEFAULT_ATOMIC; N],
            queue: BinaryHeap::new(),
        }
    }

    /// Returns the current priority of an interrupt source.
    #[inline(always)]
    pub fn get_priority<I: InterruptNumber>(&self, interrupt: I) -> u8 {
        self.priorities[interrupt.number() as usize]
    }

    /// Sets the priority of an interrupt source.
    ///
    /// # Note
    ///
    /// The 0 priority level is reserved for "never interrupt".
    ///
    /// Interrupts are queued according to their priority level when queued.
    /// Thus, if you change the priority of an interrupt while it is already queued,
    /// the pending interrupt will execute with the previous priority.
    ///
    /// # Safety
    ///
    /// Changing the priority level of an interrupt may break priority-based critical sections.
    #[inline(always)]
    pub unsafe fn set_priority<I: InterruptNumber>(&mut self, interrupt: I, priority: u8) {
        self.priorities[interrupt.number() as usize] = priority;
    }

    //// Returns current priority threshold.
    #[inline(always)]
    pub fn get_threshold(&self) -> u8 {
        self.threshold.load(Ordering::Acquire)
    }

    /// Sets the priority threshold of the controller.
    ///
    /// # Safety
    ///
    /// Changing the priority threshold may break priority-based critical sections.
    #[inline(always)]
    pub unsafe fn set_threshold(&mut self, priority: u8) {
        self.threshold.store(priority, Ordering::Release);
    }

    /// Checks if a given interrupt is pending.
    #[inline(always)]
    pub fn is_pending<I: InterruptNumber>(&mut self, interrupt: I) -> bool {
        self.pending[interrupt.number() as usize].load(Ordering::Acquire)
    }

    /// Returns `true` if the next queued interrupt can be triggered.
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        match self.queue.peek() {
            Some(&(p, _)) => p > self.threshold.load(Ordering::Acquire),
            None => false,
        }
    }

    /// Sets an interrupt source as pending.
    /// Returns `true` if a software interrupt can be automatically triggered.
    ///
    /// # Notes
    ///
    /// If interrupt priority is 0 or already pending, this request is silently ignored.
    #[inline(always)]
    pub fn pend<I: InterruptNumber>(&mut self, interrupt: I) {
        let interrupt = interrupt.number();
        let i = interrupt as usize;
        if self.priorities[i] == 0 {
            return;
        }
        // set the task to pending and push to the queue if it was not pending beforehand.
        // TODO compare_exchange returns the PREVIOUS value!
        if let Ok(false) =
            self.pending[i].compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        {
            // SAFETY: we do not allow the same task to be pending more than once
            unsafe { self.queue.push_unchecked((self.priorities[i], interrupt)) };
        }
    }

    /// Pops the pending tasks with highest priority.
    #[inline]
    pub fn pop(&mut self) -> Option<(u8, u16)> {
        match self.is_ready() {
            true => {
                // SAFETY: we know the queue is not empty
                let (priority, interrupt) = unsafe { self.queue.pop_unchecked() };
                // TODO maybe compare_exchange to make sure that it was pending?
                self.pending[interrupt as usize].store(false, Ordering::SeqCst);
                Some((priority, interrupt))
            }
            false => None,
        }
    }
}
