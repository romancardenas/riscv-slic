#![no_std]
#![no_main]

extern crate panic_halt;
use e310x::CLINT;
use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};

use riscv_rt::entry;
extern crate riscv_slic;

// generate SLIC code for this example
riscv_slic::codegen!(
    pac = e310x,
    swi = [SoftLow, SoftMedium, SoftHigh],
    backend = [hart_id = HART0]
);

use slic::Interrupt as SoftInterrupt; // Re-export of automatically generated enum of interrupts in previous macro

/// HW handler for MachineTimer interrupts triggered by CLINT.
#[allow(non_snake_case)]
#[no_mangle]
fn MachineTimer() {
    let mtimecmp = CLINT::mtimecmp0();
    let val = mtimecmp.read();
    sprintln!("--- update MTIMECMP (mtimecmp = {}) ---", val);
    mtimecmp.write(val + CLINT::freq() as u64);
    riscv_slic::pend(SoftInterrupt::SoftMedium);
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
    riscv_slic::pend(SoftInterrupt::SoftLow);
    sprintln!("  middle SoftMedium");
    riscv_slic::pend(SoftInterrupt::SoftHigh);
    sprintln!("  stop SoftMedium");
}

/// Handler for SoftLow task (low priority).
#[allow(non_snake_case)]
#[no_mangle]
fn SoftLow() {
    sprintln!("start SoftLow");
    sprintln!("stop SoftLow");
}

#[entry]
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
    riscv_slic::clear_interrupts();
    // Set priorities
    unsafe {
        riscv_slic::set_priority(SoftInterrupt::SoftLow, 1); // low priority
        riscv_slic::set_priority(SoftInterrupt::SoftMedium, 2); // medium priority
        riscv_slic::set_priority(SoftInterrupt::SoftHigh, 3); // high priority
    }
    sprintln!("Done!");

    sprintln!("Enabling interrupts...");
    unsafe {
        riscv_slic::set_interrupts();
        CLINT::mtimer_enable();
        riscv_slic::enable();
    }
    //let mut delay = CLINT::delay();
    loop {
        sprintln!("Going to sleep!");
        unsafe { riscv_slic::riscv::asm::wfi() };
    }
}
