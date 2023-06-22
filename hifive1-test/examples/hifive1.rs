#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};
use riscv_rt::entry;

// generate SLIC code for this example, only adding a HW binding for RTC
// and a purely software SoftLow interrupt
riscv_slic::codegen!(e310x, [RTC], [SoftLow, SoftHigh]);
use slic::Interrupt; // Re-export of automatically generated enum of interrupts in previous macro

/// HW handler for clearing RTC.
/// We must define a ClearX handler for every bypassed HW interrupt
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn ClearRTC() {
    sprintln!("!start ClearRTC");
    // increase rtccmp to clear HW interrupt
    let rtc = DeviceResources::steal().peripherals.RTC;
    let rtccmp = rtc.rtccmp.read().bits();
    rtc.rtccmp.write(|w| w.bits(rtccmp + 65536 * 2));
    sprintln!("!stop ClearRTC (rtccmp = {})", rtccmp);
}

/// SW handler for RTC.
/// This task is automatically pended right after executing ClearRTC.
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn RTC() {
    sprintln!("  start RTC");
    riscv_slic::pend(Interrupt::SoftLow);
    sprintln!("  middle RTC");
    riscv_slic::pend(Interrupt::SoftHigh);
    sprintln!("  stop RTC");
}

/// SW handler for SoftLow (low priority task with no HW binding).
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn SoftLow() {
    sprintln!("start SoftLow");
    sprintln!("stop SoftLow");
}

/// SW handler for SoftHigh (high priority task with no HW binding).
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn SoftHigh() {
    sprintln!("    start SoftHigh");
    sprintln!("    stop SoftHigh");
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
        riscv_slic::set_priority(Interrupt::RTC, 2); // medium priority
        riscv_slic::set_priority(Interrupt::SoftHigh, 3); // high priority
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
