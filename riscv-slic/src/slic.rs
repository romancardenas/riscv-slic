use core::cell::RefCell;
use critical_section::Mutex;
use heapless::binary_heap::{BinaryHeap, Max};

#[doc(hidden)]
pub type MutexSLIC<const N: usize> = Mutex<RefCell<SLIC<N>>>;

#[doc(hidden)]
#[inline]
pub const fn new_slic<const N: usize>() -> MutexSLIC<N> {
    Mutex::new(RefCell::new(SLIC::new()))
}

/// Software interrupt controller
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub struct SLIC<const N: usize> {
    /// priority threshold. The controller only triggers software
    /// interrupts if there is a pending interrupt with higher priority.
    threshold: u8,
    /// Array with the priorities assigned to each software interrupt source.
    /// Priority 0 is reserved for "interrupt diabled".
    priorities: [u8; N],
    /// Array to check if a software interrupt source is pending.
    pending: [bool; N],
    /// Priority queue with pending interrupt sources.
    queue: BinaryHeap<(u8, u16), Max, N>,
}

impl<const N: usize> SLIC<N> {
    /// Creates a new software interrupt controller protected by a mutex.
    #[inline]
    const fn new() -> Self {
        Self {
            threshold: 0,
            priorities: [0; N],
            pending: [false; N],
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
    #[inline]
    pub fn set_priority(&mut self, interrupt: u16, priority: u8) {
        self.priorities[interrupt as usize] = priority;
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
    /// Setting the priority threshold to a value lower than the current threshold
    /// may lead to priority inversion. If you want to make sure that the threshold
    /// is raised, use the [`raise_threshold`] method instead.
    #[inline]
    pub unsafe fn set_threshold(&mut self, priority: u8) {
        self.threshold = priority;
    }

    /// Sets the priority threshold only to a higher value than the current threshold.
    /// When the threshold is raised, the function returns `Ok(prev_threshold)`.
    /// Otherwise, the threshold is not changed and `Err(())` is returned.
    pub fn raise_threshold(&mut self, priority: u8) -> Result<u8, ()> {
        if priority > self.threshold {
            let prev = self.threshold;
            self.threshold = priority;
            Ok(prev)
        } else {
            Err(())
        }
    }

    /// Checks if a given interrupt is pending.
    #[inline]
    pub fn is_pending(&mut self, interrupt: u16) -> bool {
        self.pending[interrupt as usize]
    }

    /// Returns `true` if the next queued interrupt can be triggered.
    #[inline]
    pub fn is_ready(&self) -> bool {
        match self.queue.peek().map(|&(p, _)| p) {
            Some(p) => p > self.threshold,
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
        if !self.pending[i] {
            self.pending[i] = true;
            // SAFETY: we guarantee that the same task can not be pending more than once
            unsafe { self.queue.push_unchecked((self.priorities[i], interrupt)) };
        }
    }

    /// Pops the pending tasks with highest priority.
    #[inline]
    pub fn pop(&mut self) -> Option<(u8, u16)> {
        while self.is_ready() {
            // SAFETY: we guarantee that the queue is not empty
            let (priority, interrupt) = unsafe { self.queue.pop_unchecked() };
            let i = interrupt as usize;
            if self.pending[i] {
                self.pending[i] = false;
                return Some((priority, interrupt));
            }
        }
        None
    }
}
