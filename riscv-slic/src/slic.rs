use heapless::binary_heap::{BinaryHeap, Max};
use portable_atomic::{AtomicBool, AtomicU8, Ordering};

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
    #[inline]
    pub fn get_priority(&self, interrupt: u16) -> u8 {
        self.priorities[interrupt as usize]
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
    #[inline]
    pub unsafe fn set_priority(&mut self, interrupt: u16, priority: u8) {
        self.priorities[interrupt as usize] = priority;
    }

    //// Returns current priority threshold.
    #[inline]
    pub fn get_threshold(&self) -> u8 {
        self.threshold.load(Ordering::Acquire)
    }

    /// Sets the priority threshold of the controller.
    ///
    /// # Safety
    ///
    /// Changing the priority threshold may break priority-based critical sections.
    #[inline]
    pub unsafe fn set_threshold(&mut self, priority: u8) {
        self.threshold.store(priority, Ordering::Release);
    }

    /// Checks if a given interrupt is pending.
    #[inline]
    pub fn is_pending(&mut self, interrupt: u16) -> bool {
        self.pending[interrupt as usize].load(Ordering::Acquire)
    }

    /// Returns `true` if the next queued interrupt can be triggered.
    #[inline]
    pub fn is_ready(&self) -> bool {
        match self.queue.peek() {
            Some(&(p, _)) => p > self.threshold.load(Ordering::Acquire),
            None => false,
        }
    }

    /// Sets an interrupt source as pending.
    ///
    /// # Notes
    ///
    /// If interrupt priority is 0 or already pending, this request is silently ignored.
    #[inline]
    pub fn pend(&mut self, interrupt: u16) {
        let i = interrupt as usize;
        if self.priorities[i] == 0 {
            return;
        }
        // set the task to pending and push to the queue if it was not pending beforehand.
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
        while self.is_ready() {
            if let Some((priority, interrupt)) = self.queue.pop() {
                if let Ok(true) = self.pending[interrupt as usize].compare_exchange(
                    true,
                    false,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    return Some((priority, interrupt));
                }
            }
        }
        None
    }
}
