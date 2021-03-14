#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use samv71_hal::uart::{Serial, Config, BaudRate, Parity, ChannelMode};
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
                piod.pd26.enable(cs).into_alternate_af2(cs),
                piod.pd25.enable(cs).into_alternate_af2(cs)
            )
        });

    let config = Config::new(BaudRate(15), Parity::NoParity, ChannelMode::Normal, false);
    let mut serial2= Serial::uart2(p.UART2, pins, config);
    serial2.write(0x28);

    loop {

    }
}

