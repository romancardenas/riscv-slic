#![no_std]
#![no_main]

extern crate panic_halt;

use riscv_slic::InterruptNumber;

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
    Soft0,
    Soft1,
    Soft2,
}

/// HW handler for MachineTimer interrupts triggered by CLINT.
#[allow(static_mut_refs)]
#[riscv_rt::core_interrupt(CoreInterrupt::MachineTimer)]
fn machine_timer() {
    static mut COUNT: u32 = 0;
    unsafe {
        sprintln!("Timer IN ({})", COUNT);
        COUNT += 1;
    }

    let clint = unsafe { Clint::steal() };
    let mtimer = clint.mtimer();
    let mtimecmp = mtimer.mtimecmp_mhartid();
    mtimecmp.modify(|val| *val += mtimer.mtime_freq() as u64);

    riscv_slic::disable();
    for i in 0..=SoftInterrupt::MAX_INTERRUPT_NUMBER {
        let interrupt = SoftInterrupt::from_number(i).unwrap();
        riscv_slic::pend(interrupt);
        sprintln!("Pend: {:?}", interrupt);
    }
    unsafe { riscv_slic::enable() };

    sprintln!("Timer OUT");
}

/// Handler for Soft0 task (lowest priority).
#[riscv_slic::interrupt(SoftInterrupt::Soft0)]
fn soft0() {
    sprintln!(" +start Soft0");
    sprintln!(" -stop Soft0");
}

/// Handler for Soft1 task (medium priority).
#[riscv_slic::interrupt(SoftInterrupt::Soft1)]
fn soft1() {
    sprintln!(" +start Soft1");
    sprintln!(" -stop Soft1");
}

/// Handler for Soft2 task (high priority).
#[riscv_slic::interrupt(SoftInterrupt::Soft2)]
fn soft2() {
    sprintln!(" +start Soft2");
    sprintln!(" -stop Soft2");
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
    // First, we make sure that all PLIC the interrupts are disabled and set the interrupts priorities
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
        riscv_slic::set_priority(SoftInterrupt::Soft0, 1); // low priority
        riscv_slic::set_priority(SoftInterrupt::Soft1, 2); // medium priority
        riscv_slic::set_priority(SoftInterrupt::Soft2, 3); // high priority
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
    }
}
