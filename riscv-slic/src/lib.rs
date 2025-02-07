#![no_std]

pub use critical_section;
pub use riscv;
pub use riscv_slic_macros::*;

mod api;
mod slic;

pub use api::*;
pub use slic::{new_slic, MutexSLIC};

/// Trait for enums of software interrupt numbers.
///
/// This trait should only be implemented by the [`riscv_slic_macros::codegen`]
/// macro for the enum of available software interrupts.
/// Each variant must convert to a `u16` of its interrupt number.
///
/// # Safety
///
/// Do NOT implement this trait. It is left for [`riscv_slic_macros::codegen`].
/// This trait must only be implemented on enums of software interrupts. Each
/// enum variant must represent a distinct value (no duplicates are permitted),
/// and must always return the same value (do not change at runtime).
/// All the interrupt numbers must be less than or equal to `MAX_INTERRUPT_NUMBER`.
/// `MAX_INTERRUPT_NUMBER` must coincide with the highest allowed interrupt number.
///
/// These requirements ensure safe nesting of critical sections.
pub unsafe trait InterruptNumber: Copy {
    /// Highest number assigned to an interrupt source.
    const MAX_INTERRUPT_NUMBER: u16;

    /// Converts an interrupt source to its corresponding number.
    fn number(self) -> u16;

    /// Tries to convert a number to a valid interrupt source.
    /// If the conversion fails, it returns an error with the number back.
    fn from_number(value: u16) -> Result<Self, u16>;
}
