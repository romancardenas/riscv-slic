#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};
use riscv::register::{mie, mstatus};
use riscv_rt::entry;
use riscv_slic;

use hifive1::hal::e310x;

// generate SLIC code for this example, only adding a HW interrupt for
// GPIO4
riscv_slic::codegen!(e310x, [RTC], []);

// Recursive expansion of codegen! macro
// ======================================

// create a handler for GPIO4
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn RTC() {
    sprintln!("Hooray! We reached RTC interrupt.");
    sprintln!("We got here with a priority of: ");
    unsafe {
        let prio = slic::__slic_get_priority(e310x::Interrupt::GPIO4 as u16);
        sprintln!("{0}", prio);
    }
    slic::clear_interrupt();
}

#[entry]
fn main() -> ! {
    let dr = unsafe { DeviceResources::steal() };

    let p = dr.peripherals;
    let pins = dr.pins;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 64.mhz().into());

    // make sure that interrupts are off
    let mut plic = e310x::PLIC::new();
    unsafe {
        mstatus::clear_mie();
        mie::clear_mtimer();
        mie::clear_mext();
        plic.reset();
    };

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    unsafe {
        // slic::enable();
        slic::__slic_set_threshold(5);
        slic::__slic_set_priority(e310x::Interrupt::RTC, 2);
        slic::enable();
    }

    // Configure RTC
    let mut rtc = p.RTC.constrain();
    rtc.disable();
    rtc.set_scale(0);
    rtc.set_rtc(0);
    rtc.set_rtccmp(10000);

    // activate interrupts
    unsafe {
        //mie::set_mext();
        //mie::set_mtimer();
        mstatus::set_mie();
        rtc.enable();
    };

    /*     let pin_gpio4 = pin!(pins, dig12);
    pin_gpio4.into_pull_up_input();
    let mut plic = PLIC::new();
    unsafe {
        plic.reset();
    };

    sprintln!("Setting up the SLIC...");
    unsafe {
        slic::enable();
        mstatus::set_mie();
        sprintln!("Some threshold tests...");
        // slic::__slic_set_threshold(5);
        let thresh = slic::__slic_get_threshold();
        sprintln!("Current threshold: {0:?}", thresh);
        sprintln!("Setting some threshold...");
        slic::__slic_set_threshold(5);
        let thresh = slic::__slic_get_threshold();
        sprintln!("New threshold: {0:?}", thresh);
        sprintln!("Setting up interrupt for GPIO4");
        slic::__slic_set_priority(e310x::Interrupt::GPIO4, 6);
        sprintln!("Ready, now interrupt pin 12!");
        // Now wait for the interrupt to come...
    } */
    // Disable watchdog
    let wdg = p.WDOG;
    wdg.wdogcfg.modify(|_, w| w.enalways().clear_bit());
    loop {}
}
