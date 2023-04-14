#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};
use riscv_rt::entry;

// generate SLIC code for this example, only adding a HW binding for RTC
// and a purely software SoftLow interrupt
riscv_slic::codegen!(e310x, [RTC], [SoftLow]);
use slic::Interrupt; // Re-export of automatically generated enum of interrupts in previous macro

/// HW handler for clearing RTC.
/// We must define a ClearX handler for every bypassed HW interrupt
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn ClearRTC() {
    // increase rtccmp to clear HW interrupt
    let rtc = DeviceResources::steal().peripherals.RTC;
    let rtccmp = rtc.rtccmp.read().bits();
    sprintln!("clear RTC (rtccmp = {})", rtccmp);
    rtc.rtccmp.write(|w| w.bits(rtccmp + 65536));
    // we also pend the lowest priority SW task before the RTC SW task is automatically pended
    riscv_slic::pend(Interrupt::SoftLow);
}

/// SW handler for RTC.
/// This task is automatically pended right after executing ClearRTC.
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn RTC() {
    sprintln!("software RTC");
}

/// SW handler for SoftLow (low priority task with no HW binding).
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn SoftLow() {
    sprintln!("software SoftLow");
}

#[entry]
fn main() -> ! {
    let dr = DeviceResources::take().unwrap();
    let p = dr.peripherals;
    let pins = dr.pins;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 64.mhz().into());

    // make sure that interrupts are off
    unsafe {
        riscv_slic::disable();
        riscv_slic::clear_interrupts();
    };

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    // Disable watchdog
    let wdg = p.WDOG;
    wdg.wdogcfg.modify(|_, w| w.enalways().clear_bit());

    // Configure SLIC
    unsafe {
        riscv_slic::set_priority(Interrupt::SoftLow, 1); // low priority
        riscv_slic::set_priority(Interrupt::RTC, 2); // high priority
    }

    // Configure RTC
    let mut rtc = p.RTC.constrain();
    rtc.disable();
    rtc.set_scale(0);
    rtc.set_rtc(0);
    rtc.set_rtccmp(10000);
    rtc.enable();

    // activate interrupts
    unsafe {
        riscv_slic::set_interrupts();
        riscv_slic::enable();
    };

    loop {}
}
