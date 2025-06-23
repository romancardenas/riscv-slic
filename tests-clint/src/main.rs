#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::{
    hal::{
        DeviceResources,
        e310x::{self, Clint},
        prelude::*,
    },
    pin, sprintln,
};

#[derive(Debug, Clone, Copy)]
#[riscv_slic::swi(pac = e310x)]
enum SoftInterrupt {
    Low,
    Medium,
    High,
}

/// HW handler for MachineTimer interrupts triggered by CLINT.
#[riscv_rt::core_interrupt(CoreInterrupt::MachineTimer)]
fn machine_timer() {
    let clint = unsafe { Clint::steal() };
    let mtimer = clint.mtimer();
    let mtimecmp = mtimer.mtimecmp_mhartid();
    mtimecmp.modify(|val| *val += mtimer.mtime_freq() as u64);
}

/// Handler for SoftHigh task (high priority).
#[riscv_slic::interrupt(SoftInterrupt::High)]
fn high() {
    sprintln!("    start SoftHigh");
    sprintln!("    stop SoftHigh");
}

/// Handler for SoftMedium task (medium priority). This task pends both SoftLow and SoftHigh.
#[riscv_slic::interrupt(SoftInterrupt::Medium)]
fn medium() {
    sprintln!("  start SoftMedium");
    riscv_slic::pend(SoftInterrupt::Low);
    sprintln!("  middle SoftMedium");
    riscv_slic::pend(SoftInterrupt::High);
    sprintln!("  stop SoftMedium");
}

/// Handler for SoftLow task (low priority).
#[riscv_slic::interrupt(SoftInterrupt::Low)]
fn low() {
    sprintln!("start SoftLow");
    sprintln!("stop SoftLow");
}

#[riscv_rt::entry]
fn main() -> ! {
    let resources = DeviceResources::take().unwrap();
    let core_peripherals = resources.core_peripherals;
    let peripherals = resources.peripherals;

    let clocks = hifive1::configure_clocks(peripherals.PRCI, peripherals.AONCLK, 64.mhz().into());
    let gpio = resources.pins;

    // Configure UART for stdout
    hifive1::stdout::configure(
        peripherals.UART0,
        pin!(gpio, uart0_tx),
        pin!(gpio, uart0_rx),
        115_200.bps(),
        clocks,
    );

    sprintln!("Configuring CLINT...");
    let clint = core_peripherals.clint;
    // First, we make sure that all PLIC the interrupts are disabled and set the interrupts priorities
    clint.disable();
    let mtimer = clint.mtimer();
    mtimer
        .mtimecmp_mhartid()
        .write(clint.mtimer().mtime_freq() as u64);
    mtimer.mtime().write(0);

    sprintln!("Configuring SLIC...");
    // make sure that interrupts are off
    riscv_slic::disable();
    // Set priorities
    unsafe {
        riscv_slic::set_priority(SoftInterrupt::Low, 1); // low priority
        riscv_slic::set_priority(SoftInterrupt::Medium, 2); // medium priority
        riscv_slic::set_priority(SoftInterrupt::High, 3); // high priority
    }

    sprintln!("Enabling interrupts...");
    unsafe {
        mtimer.enable();
        riscv_slic::enable();
    }

    sprintln!("Done!");

    loop {
        sprintln!("Waiting for interrupts...");
        riscv_slic::riscv::asm::wfi();
        sprintln!("Interrupt received!");
        riscv_slic::pend(SoftInterrupt::Medium);
        sprintln!();
    }
}
