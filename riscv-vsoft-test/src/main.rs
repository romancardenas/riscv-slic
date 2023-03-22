//#![no_std]

riscv_vsoft_macros::codegen!(GPIO0, GPIO1, SPI5);

mod implements {
    #[no_mangle]
    #[allow(non_snake_case)]
    pub fn GPIO0() {}

    #[no_mangle]
    #[allow(non_snake_case)]
    pub fn GPIO1() {}

    #[no_mangle]
    #[allow(non_snake_case)]
    pub fn SPI5() {}
}

fn main() {
    assert_eq!(0, Interrupt::GPIO0 as u16);
    assert_eq!(1, Interrupt::GPIO1 as u16);
    assert_eq!(2, Interrupt::SPI5 as u16);
}
