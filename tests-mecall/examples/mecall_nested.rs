#![no_std]
#![no_main]

extern crate panic_halt;

use riscv_slic::{codegen as riscv_slic_codegen, InterruptNumber};

use hifive1::{
    hal::{
        e310x::{self, CLINT},
        prelude::*,
        DeviceResources,
    },
    pin, sprintln,
};

// generate SLIC code for this example
riscv_slic_codegen!(pac = e310x, swi = [Soft0, Soft1, Soft2]);
use slic::SoftwareInterrupt; // Re-export of automatically generated enum of interrupts in previous macro

/// HW handler for MachineTimer interrupts triggered by CLINT.
#[riscv_rt::core_interrupt(CoreInterrupt::MachineTimer)]
fn machine_timer() {
    static mut COUNT: u32 = 0;
    unsafe {
        sprintln!("Timer IN ({})", COUNT);
        COUNT += 1;
    }

    let mtimecmp = CLINT::mtimecmp0();
    mtimecmp.modify(|val| *val += CLINT::freq() as u64);

    riscv_slic::clear_interrupts();
    for i in 0..=SoftwareInterrupt::MAX_INTERRUPT_NUMBER {
        let interrupt = SoftwareInterrupt::from_number(i).unwrap();
        riscv_slic::pend(interrupt);
        sprintln!("Pend: {:?}", interrupt);
    }
    unsafe { riscv_slic::set_interrupts() };

    sprintln!("Timer OUT");
}

/// Handler for Soft0 task (lowest priority).
#[allow(non_snake_case)]
#[no_mangle]
fn Soft0() {
    sprintln!(" +start Soft0");
    sprintln!(" -stop Soft0");
}

/// Handler for Soft1 task (medium priority).
#[allow(non_snake_case)]
#[no_mangle]
fn Soft1() {
    sprintln!(" +start Soft1");
    sprintln!(" -stop Soft1");
}

/// Handler for Soft2 task (high priority).
#[allow(non_snake_case)]
#[no_mangle]
fn Soft2() {
    sprintln!(" +start Soft2");
    sprintln!(" -stop Soft2");
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
    riscv_slic::clear_interrupts();
    // Set priorities
    unsafe {
        riscv_slic::set_priority(SoftwareInterrupt::Soft0, 1); // low priority
        riscv_slic::set_priority(SoftwareInterrupt::Soft1, 2); // medium priority
        riscv_slic::set_priority(SoftwareInterrupt::Soft2, 3); // high priority
    }

    sprintln!("Enabling interrupts...");
    unsafe {
        riscv_slic::set_interrupts();
        CLINT::mtimer_enable();
        riscv_slic::enable();
    }

    sprintln!("Done!");

    loop {
        sprintln!("Waiting for interrupts...");
        riscv_slic::riscv::asm::wfi();
        sprintln!("Interrupt received!");
    }
}
