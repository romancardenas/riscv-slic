#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};
use riscv::register::{mie, mstatus};
use riscv_rt::entry;
use riscv_slic;

// generate SLIC code for this example, only adding a HW interrupt for
// GPIO4
riscv_slic::codegen!(e310x, [GPIO4], []);

// create a handler for GPIO4
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn GPIO4() {
    sprintln!("Hooray! We reached GPIO4 interrupt.");
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

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );
    let pin_gpio4 = pin!(pins, dig12);
    pin_gpio4.into_pull_up_input();

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
        slic::__slic_set_priority(e310x::Interrupt::GPIO4 as u16, 1);
        sprintln!("Ready, now interrupt pin 12!");
        // Now wait for the interrupt to come...
    }
    // Disable watchdog
    let wdg = p.WDOG;
    wdg.wdogcfg.modify(|_, w| w.enalways().clear_bit());
    loop {}
}
