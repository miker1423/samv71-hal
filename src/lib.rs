#![no_std]

use embedded_hal::serial::{Read, Write};
use atsamv71q21::UART0;
use atsamv71q21::uart0::mr::{CHMODE_A, PAR_A};

struct BaudRate(u16);

struct Serial<INTERFACE> { 
    interface: INTERFACE
}

enum Parity {
    Even, 
    Odd, 
    Mark,
    Space,
    NoParity
}

enum ChannelMode {
    Normal,
    Automatic,
    LocalLoopback,
    RemoteLoopback
}

struct Config {
    baud_rate: BaudRate,
    parity: Parity,
    channel_mode: ChannelMode,
    digital_filter: bool
}

enum UartError {
    Parity,
    Framing,
    Overrun,
}

impl Read<u8> for Serial<UART0> {
    type Error = UartError;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let uart = &self.interface;
        let status_register = &uart.sr;
        if status_register.read().rxrdy().bit_is_set() {
            let value = uart.rhr.read().rxchr().bits();
            Ok(value)
        }

        Err(nb::Error::Other(UartError::Overrun))
    }
}

impl Write<u8> for Serial<UART0> {
    type Error = UartError;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let uart = &self.interface;
        let status_register = &uart.sr.read();
        
        if status_register.ovre() {
            Err(nb::Error::Other(UartError::Overrun))
        } else if status_register.frame() {
            Err(nb::Error::Other(UartError::Framing))
        } else if status_register.pare() {
            Err(nb::Error::Other(UartError::Parity))
        }
         else if status_register.txrdy().bit_is_set() {
            uart.thr.write_with_zero(|w| unsafe { w.txchr().bits(word) });
            Ok(())
        } else {
             nb::Error::WouldBlock
         }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        let uart = &self.interface;
        let status_register = &uart.sr;
        while status_register.read().txempty().bit_is_set() {
            cortex_m::asm::nop();
        }
        Ok(())
    }
}

trait UartConfig<UART> {
    fn set_config(&self, config: Config);
}

impl UartConfig<UART0> for UART0 {
    fn set_config(&self, config:Config) {
        self.cr.write_with_zero(|w| w.txen().set_bit().rxen().set_bit());
        let variant = get_mode_variant(config.channel_mode);
        let parity = get_parity(config.parity);
        self.mr.write_with_zero(|w|
            w.chmode().variant(variant)
             .par().variant(parity)
             .filter().bit(config.digital_filter)
        );
        self.brgr.write_with_zero(|w| unsafe { w.bits(config.baud_rate.0 as u32) });
    }
}

fn get_mode_variant(mode: ChannelMode) -> CHMODE_A {
    match mode {
        ChannelMode::Normal => CHMODE_A::NORMAL,
        ChannelMode::Automatic => CHMODE_A::AUTOMATIC,
        ChannelMode::LocalLoopback => CHMODE_A::LOCAL_LOOPBACK,
        ChannelMode::RemoteLoopback => CHMODE_A::REMOTE_LOOPBACK
    }
}

fn get_parity(parity: Parity) -> PAR_A {
    match parity {
        Parity::Even => PAR_A::EVEN,
        Parity::Mark => PAR_A::MARK,
        Parity::NoParity => PAR_A::NO,
        Parity::Odd => PAR_A::ODD,
        Parity::Space => PAR_A::SPACE
    }
}


/*
trait UartConfig {
    fn set_baudrate(&self, baud_rate: BaudRate);
    fn set_parity(&self, parity: Parity);
    fn set_channel_mode(&self, channel_mode: ChannelMode);
    fn enable_digital_filter(&self);
    fn disable_digital_filter(&self);
}
*/

#[cfg(test)]
mod tests {
}
