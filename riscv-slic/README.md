[![crates.io](https://img.shields.io/crates/d/riscv-slic.svg)](https://crates.io/crates/riscv-slic)
[![crates.io](https://img.shields.io/crates/v/riscv-slic.svg)](https://crates.io/crates/riscv-slic)

# `riscv-slic`

Crate for enabling vectored handling of software interrupts for RISC-V targets inspired by PLIC.

This crate creates a software interrupt vector with as many interrupt sources as requested by the user.
Each software interrupt source can be enabled/disabled independently, and you can assign a different priority level to each of them.
Priority level 0 is reserved to disable the interrupt. By default, all the software interrupt sources are set to priority level 0.
The maximum allowed priority level is 255.
Additionally, you can set a software interrupt priority threshold.
Only interrupt sources with a priority level above the threshold will cause interrupts.
A threshold of 0 means that all the active interrupt sources can cause an interrupt.
Alternatively, a threshold of 255 implies that none of the interrupt sources will cause an interrupt.

If you pend a software interrupt source with a priority higher than the current threshold, it will cause a software interrupt in your RISC-V processor.
How software interrupts are triggered depends on your target, and you need to activate a proper feature when compiling this crate.
If your target has a CLINT peripheral, you can activate the `clint-backend` feature.
Currently, this is the only supported way to use this crate.
Open an issue or a RFC in GitHub if you would like other particular target to work with `riscv-slic`.


## [Documentation](https://docs.rs/crate/riscv-slic)

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.75 and up.
It *might* compile with older versions but that may change in any new patch release.
