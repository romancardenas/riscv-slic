#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::e310x::{Interrupt, Priority};
use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};
use riscv::register::{mie, mstatus};
use riscv_rt::entry;

// generate SLIC code for this example, only adding a HW binding for RTC
riscv_slic::codegen!(e310x, [], [SoftRTC]);
// TODO molaría hacer PAC, Software, [(Hardware,ClearPart,SoftPart)]

/// HW handler for RTC
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn RTC() {
    // increase rtccmp to clear HW interrupt
    let rtc = DeviceResources::steal().peripherals.RTC;
    let rtccmp = rtc.rtccmp.read().bits();
    sprintln!("external RTC (rtccmp = {})", rtccmp);
    rtc.rtccmp.write(|w| w.bits(rtccmp + 65536));
    // pend corresponding software task
    slic::pend(slic::Interrupt::SoftRTC);
}

/// SW handler for RTC
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn SoftRTC() {
    sprintln!("software");
}

#[entry]
fn main() -> ! {
    let dr = DeviceResources::take().unwrap();

    let cp = dr.core_peripherals;
    let p = dr.peripherals;
    let pins = dr.pins;
    let mut plic = cp.plic;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 64.mhz().into());

    // make sure that interrupts are off
    unsafe {
        plic.reset();
        mstatus::clear_mie();
        mie::clear_mtimer();
        mie::clear_mext();
    };

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    // Configure RTC
    let mut rtc = p.RTC.constrain();
    rtc.disable();
    rtc.set_scale(0);
    rtc.set_rtc(0);
    rtc.set_rtccmp(10000);

    // Disable watchdog
    let wdg = p.WDOG;
    wdg.wdogcfg.modify(|_, w| w.enalways().clear_bit());

    // Configure SLIC
    unsafe {
        slic::set_priority(slic::Interrupt::SoftRTC, 2);
        slic::set_threshold(0);
    }

    // Configure PLIC
    unsafe {
        plic.enable_interrupt(Interrupt::RTC);
        plic.set_priority(Interrupt::RTC, Priority::P1);
        plic.set_threshold(Priority::P0);
    }

    // activate interrupts
    unsafe {
        mie::set_mext();
        mie::set_msoft();
        mstatus::set_mie();
        rtc.enable();
    };

    loop {}
}
