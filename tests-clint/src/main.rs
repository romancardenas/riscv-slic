#![no_std]
#![no_main]

extern crate panic_halt;
extern crate riscv_slic;

use hifive1::{
    hal::{
        e310x::{self, CLINT},
        prelude::*,
        DeviceResources,
    },
    pin, sprintln,
};

// generate SLIC code for this example
riscv_slic::codegen!(
    pac = e310x,
    swi = [SoftLow, SoftMedium, SoftHigh],
    backend = [hart_id = H0]
);
use slic::SoftwareInterrupt; // Re-export of automatically generated enum of interrupts in previous macro

/// HW handler for MachineTimer interrupts triggered by CLINT.
#[riscv_rt::core_interrupt(CoreInterrupt::MachineTimer)]
fn machine_timer() {
    let mtimecmp = CLINT::mtimecmp0();
    mtimecmp.modify(|val| *val += CLINT::freq() as u64);
}

/// Handler for SoftHigh task (high priority).
#[allow(non_snake_case)]
#[no_mangle]
fn SoftHigh() {
    sprintln!("    start SoftHigh");
    sprintln!("    stop SoftHigh");
}

/// Handler for SoftMedium task (medium priority). This task pends both SoftLow and SoftHigh.
#[allow(non_snake_case)]
#[no_mangle]
fn SoftMedium() {
    sprintln!("  start SoftMedium");
    riscv_slic::pend(SoftwareInterrupt::SoftLow);
    sprintln!("  middle SoftMedium");
    riscv_slic::pend(SoftwareInterrupt::SoftHigh);
    sprintln!("  stop SoftMedium");
}

/// Handler for SoftLow task (low priority).
#[allow(non_snake_case)]
#[no_mangle]
fn SoftLow() {
    sprintln!("start SoftLow");
    sprintln!("stop SoftLow");
}

#[riscv_rt::entry]
fn main() -> ! {
    let resources = DeviceResources::take().unwrap();
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
    // First, we make sure that all PLIC the interrupts are disabled and set the interrupts priorities
    CLINT::disable();
    let mtimer = CLINT::mtimer();
    mtimer.mtimecmp0.write(CLINT::freq() as u64);
    mtimer.mtime.write(0);

    sprintln!("Configuring SLIC...");
    // make sure that interrupts are off
    riscv_slic::disable();
    // Set priorities
    unsafe {
        riscv_slic::set_priority(SoftwareInterrupt::SoftLow, 1); // low priority
        riscv_slic::set_priority(SoftwareInterrupt::SoftMedium, 2); // medium priority
        riscv_slic::set_priority(SoftwareInterrupt::SoftHigh, 3); // high priority
    }

    sprintln!("Enabling interrupts...");
    unsafe {
        CLINT::mtimer_enable();
        riscv_slic::enable();
    }

    sprintln!("Done!");

    loop {
        sprintln!("Waiting for interrupts...");
        riscv_slic::riscv::asm::wfi();
        sprintln!("Interrupt received!");
        riscv_slic::pend(SoftwareInterrupt::SoftMedium);
        sprintln!();
    }
}
