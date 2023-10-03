#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::e310x::{Interrupt as ExtInterrupt, Priority, PLIC};
use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};

use riscv_rt::entry;
use riscv_slic;

// generate SLIC code for this example
riscv_slic::codegen!(
    pac = e310x,
    swi = [SoftLow, SoftMedium, SoftHigh],
    backend = [hart_id = HART0]
);
use slic::Interrupt as SoftInterrupt; // Re-export of automatically generated enum of interrupts in previous macro

/// HW handler for clearing RTC.
/// We must define a ClearX handler for every bypassed HW interrupt
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn RTC() {
    // increase rtccmp to clear HW interrupt
    let rtc = DeviceResources::steal().peripherals.RTC;
    let rtccmp = rtc.rtccmp.read().bits();
    sprintln!("--- clear RTC (rtccmp = {}) ---", rtccmp);
    rtc.rtccmp.write(|w| w.bits(rtccmp + 65536));
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

    // Disable watchdog
    let wdg = peripherals.WDOG;
    wdg.wdogcfg.modify(|_, w| w.enalways().clear_bit());

    sprintln!("Configuring PLIC!!!...");
    // First, we make sure that all PLIC the interrupts are disabled and set the interrupts priorities
    PLIC::disable();
    PLIC::priorities().reset::<ExtInterrupt>();
    // Safety: interrupts are disabled
    unsafe { PLIC::priorities().set_priority(ExtInterrupt::RTC, Priority::P7) };

    // Next, we configure the PLIC context for our use case
    let ctx = PLIC::ctx0();
    ctx.enables().disable_all::<ExtInterrupt>();
    // Safety: we are the only hart running and we have not enabled any interrupts yet
    unsafe {
        ctx.enables().enable(ExtInterrupt::RTC);
        ctx.threshold().set_threshold(Priority::P1);
    };
    sprintln!("done!");

    sprintln!("Configuring RTC...");
    let mut rtc = peripherals.RTC.constrain();
    rtc.disable();
    rtc.set_scale(0);
    rtc.set_rtc(0);
    rtc.set_rtccmp(10000);
    rtc.enable();
    sprintln!("done!");

    sprintln!("Configuring SLIC...");
    // make sure that interrupts are off
    unsafe { riscv_slic::disable() };
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
        PLIC::enable();
        riscv_slic::enable();
    }
    loop {
        sprintln!("Going to sleep!");
        unsafe { riscv_slic::riscv::asm::wfi() };
    }
}
