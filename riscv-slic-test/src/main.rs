//#![no_std]

riscv_slic::codegen!(e310x, [GPIO0, GPIO1, SPI5]);

#[no_mangle]
#[allow(non_snake_case)]
pub fn GPIO0() {}

#[no_mangle]
#[allow(non_snake_case)]
pub fn GPIO1() {}

#[no_mangle]
#[allow(non_snake_case)]
pub fn SPI5() {}

fn main() {
    assert_eq!(0, slic::Interrupt::GPIO0 as u16);
    assert_eq!(1, slic::Interrupt::GPIO1 as u16);
    assert_eq!(2, slic::Interrupt::SPI5 as u16);
}
