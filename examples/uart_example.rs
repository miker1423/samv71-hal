#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use samv71_hal::serial::uart::{Serial, Config, Parity, ChannelMode};
use samv71_hal::serial::BaudRate;
use samv71_hal::gpio::GpioExt;
use samv71_hal::pac as sam;
use samv71_hal::prelude::*;


#[entry]
fn main() -> ! {
    let p = sam::Peripherals::take().unwrap();
    let piod = p.PIOD.split();
    let pins = cortex_m::interrupt::free(move |cs|
        {
            (
                piod.pd26.into_alternate_af2(cs).disable(cs),
                piod.pd25.into_alternate_af2(cs).disable(cs)
            )
        });

    let config = Config::new(9600.into(), Parity::NoParity, ChannelMode::Normal, false);
    let mut serial2= Serial::uart2(p.UART2, pins, config, &p.PMC);
    serial2.write(0x28);

    loop {

    }
}

