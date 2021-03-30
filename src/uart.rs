use embedded_hal::serial::{Read, Write};
use core::convert::{ Infallible, Into };
use core::marker::PhantomData;
use crate::gpio::*;
use crate::pac::PMC;

pub enum Parity {
    Even,
    Odd,
    Mark,
    Space,
    NoParity
}

pub enum ChannelMode {
    Normal,
    Automatic,
    LocalLoopback,
    RemoteLoopback
}

pub enum UartError {
    Parity,
    Framing,
    Overrun,
}

pub struct BaudRate(pub u16);

impl Into<BaudRate> for u16 {
    fn into(self) -> BaudRate {
        BaudRate(self)
    }
}

pub trait RxPin<UART> {}
pub trait TxPin<UART> {}

macro_rules! uart_pins {
    ($($UART:ident => {
        tx => [$($tx:ty),+ $(,)*],
        rx => [$($rx:ty),+ $(,)*],
    })+) => {
        $(
            $(
                impl TxPin<crate::pac::$UART> for $tx {}
            )+
            $(
                impl RxPin<crate::pac::$UART> for $rx {}
            )+
        )+
    }
}

uart_pins! {
    UART0 => {
        tx => [pioa::PA10<Alternate<AF0>>],
        rx => [pioa::PA9<Alternate<AF0>>],
    }
    UART1 => {
        tx => [pioa::PA4<Alternate<AF2>>, pioa::PA6<Alternate<AF2>>, piod::PD26<Alternate<AF3>>],
        rx => [pioa::PA5<Alternate<AF2>>],
    }
    UART2 => {
        tx => [piod::PD26<Alternate<AF2>>],
        rx => [piod::PD25<Alternate<AF2>>],
    }
    UART3 => {
        tx => [piod::PD30<Alternate<AF0>>, piod::PD31<Alternate<AF1>>],
        rx => [piod::PD28<Alternate<AF0>>],
    }
    UART4 => {
        tx => [piod::PD19<Alternate<AF2>>, piod::PD3<Alternate<AF2>>],
        rx => [piod::PD18<Alternate<AF2>>],
    }
}

pub struct Rx<UART> {
    _instance: PhantomData<UART>,
}

pub struct Tx<UART> {
    _instance: PhantomData<UART>,
}

pub struct Serial<UART, TXPIN, RXPIN> {
    uart: UART,
    pins: (TXPIN, RXPIN),
}

impl<UART, TXPIN, RXPIN> Serial<UART, TXPIN, RXPIN>
{
    pub fn split(self) -> (Tx<UART>, Rx<UART>)
        where
            TXPIN: TxPin<UART>,
            RXPIN: RxPin<UART>,
    {
        (
            Tx {
                _instance: PhantomData,
            },
            Rx {
                _instance: PhantomData,
            }
        )
    }

    pub fn release(self) -> (TXPIN, RXPIN) {
        self.pins
    }
}



pub struct Config {
    baud_rate: BaudRate,
    parity: Parity,
    channel_mode: ChannelMode,
    digital_filter: bool
}

impl Config {
    pub fn new(baud_rate: BaudRate, parity: Parity, channel_mode: ChannelMode, digital_filter: bool) -> Config {
        Config {baud_rate, parity, channel_mode, digital_filter}
    }
}

trait ConfigMethod {
    type Parity;
    type Mode;

    fn get_parity(&self, parity: &Parity) -> Self::Parity;

    fn get_mode(&self, mode: &ChannelMode) -> Self::Mode;

    fn configure(&self, config: Config, pmc: &PMC);
}

macro_rules! uart {
    ($($UART:ident: ($uart:ident, $uarttx: ident, $uartrx:ident, $pmc_pcerx:ident, $pid:ident),)+) => {
        $(
            use crate::pac::$UART;

            impl<TXPIN, RXPIN> Serial<$UART, TXPIN, RXPIN>
            where
                TXPIN: TxPin<$UART>,
                RXPIN: RxPin<$UART>,
            {
                pub fn $uart(uart: $UART, pins: (TXPIN, RXPIN), config: Config, pmc: &PMC) -> Self {
                    let serial = Serial { uart, pins };
                    serial.configure(config, pmc);
                    serial.uart.cr.write_with_zero(|w| w.txen().set_bit().rxen().set_bit());
                    serial
                }
            }

            impl core::fmt::Write for Tx<$UART>
                where
                    Tx<$UART>: embedded_hal::serial::Write<u8>,
            {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    s.as_bytes()
                        .iter()
                        .try_for_each(|c| nb::block!(self.write(*c)))
                        .map_err(|_| core::fmt::Error)
                }
            }

            impl<TXPIN, RXPIN> core::fmt::Write for Serial<$UART, TXPIN, RXPIN>
                where
                    TXPIN: TxPin<$UART>,
            {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    s.as_bytes()
                        .iter()
                        .try_for_each(|c| nb::block!(self.write(*c)))
                        .map_err(|_| core::fmt::Error)
                }
            }

            impl<TXPIN, RXPIN> Read<u8> for Serial<$UART, TXPIN, RXPIN>
                where
                RXPIN: RxPin<$UART>
            {
                type Error = UartError;

                fn read(&mut self) -> nb::Result<u8, Self::Error> {
                    let status_register = unsafe { (&*$UART::ptr()).sr.read() };
                    if status_register.ovre().bit() {
                        Err(nb::Error::Other(UartError::Overrun))
                    } else if status_register.frame().bit() {
                        Err(nb::Error::Other(UartError::Framing))
                    } else if status_register.pare().bit() {
                        Err(nb::Error::Other(UartError::Parity))
                    } else if status_register.rxrdy().bit() {
                        let rhr = unsafe { (&*$UART::ptr()).rhr.read() };
                        let value = rhr.rxchr().bits();
                        Ok(value)
                    } else {
                        nb::Result::Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl Read<u8> for Rx<$UART>
            {
                type Error = UartError;

                fn read(&mut self) -> nb::Result<u8, Self::Error>
                {
                    let status_register = unsafe { (&*$UART::ptr()).sr.read() };
                    if status_register.ovre().bit() {
                        Err(nb::Error::Other(UartError::Overrun))
                    } else if status_register.frame().bit() {
                        Err(nb::Error::Other(UartError::Framing))
                    } else if status_register.pare().bit() {
                        Err(nb::Error::Other(UartError::Parity))
                    } else if status_register.rxrdy().bit() {
                        let rhr = unsafe { (&*$UART::ptr()).rhr.read() };
                        let value = rhr.rxchr().bits();
                        Ok(value)
                    } else {
                        nb::Result::Err(nb::Error::WouldBlock)
                    }
                }
            }


            impl<TXPIN, RXPIN> Write<u8> for Serial<$UART, TXPIN, RXPIN>
                where
                    TXPIN: TxPin<$UART>,
            {
                type Error = Infallible;

                fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$UART::ptr()).sr.read() };
                    if status_register.txrdy().bit_is_set() {
                        let uart = unsafe { (&*$UART::ptr()) };
                        uart.thr.write_with_zero(|w| unsafe { w.txchr().bits(byte) });
                        nb::Result::Ok(())
                    } else {
                        nb::Result::Err(nb::Error::WouldBlock)
                    }
                }

                fn flush(&mut self) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$UART::ptr()).sr.read() };
                    if status_register.txempty().bit_is_set() {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl Write<u8> for Tx<$UART>
            {
                type Error = Infallible;

                fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$UART::ptr()).sr.read() };
                    if status_register.txrdy().bit_is_set() {
                        let uart = unsafe { (&*$UART::ptr()) };
                        uart.thr.write_with_zero(|w| unsafe { w.txchr().bits(byte) });
                        nb::Result::Ok(())
                    } else {
                        nb::Result::Err(nb::Error::WouldBlock)
                    }
                }

                fn flush(&mut self) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$UART::ptr()).sr.read() };
                    if status_register.txempty().bit_is_set() {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }


            impl<TXPIN> Serial<$UART, TXPIN, ()>
            where
                TXPIN: TxPin<$UART>,
            {
                pub fn $uarttx(uart: $UART, txpin: TXPIN, config: Config, pmc: &PMC) -> Self {
                    let rxpin = ();
                    let serial = Serial { uart, pins: (txpin, rxpin) };
                    serial.configure(config, pmc);
                    serial.uart.cr.write_with_zero(|w| w.txen().set_bit());
                    serial
                }
            }

            impl<RXPIN> Serial<$UART, (), RXPIN>
            where
                RXPIN: RxPin<$UART>
            {
                pub fn $uartrx(uart: $UART, rxpin: RXPIN, config: Config, pmc: &PMC) -> Self {
                    let txpin = ();
                    let serial = Serial { uart, pins: (txpin, rxpin)};
                    serial.configure(config, pmc);
                    serial.uart.cr.write_with_zero(|w| w.rxen().set_bit());
                    serial
                }
            }

            impl<TXPIN, RXPIN> ConfigMethod for Serial<$UART, TXPIN, RXPIN> {
                type Parity = crate::pac::$uart::mr::PAR_A;
                type Mode = crate::pac::$uart::mr::CHMODE_A;

                fn get_mode(&self, mode: &ChannelMode) -> Self::Mode {
                    match *mode {
                        ChannelMode::Normal => Self::Mode::NORMAL,
                        ChannelMode::Automatic => Self::Mode::AUTOMATIC,
                        ChannelMode::LocalLoopback => Self::Mode::LOCAL_LOOPBACK,
                        ChannelMode::RemoteLoopback => Self::Mode::REMOTE_LOOPBACK
                    }
                }


                fn get_parity(&self, parity: &Parity) -> Self::Parity {
                    match *parity {
                        Parity::Even => Self::Parity::EVEN,
                        Parity::Mark => Self::Parity::MARK,
                        Parity::NoParity => Self::Parity::NO,
                        Parity::Odd => Self::Parity::ODD,
                        Parity::Space => Self::Parity::SPACE
                    }
                }

                fn configure(&self, config: Config, pmc: &PMC) {
                    let uart = &self.uart;
                    pmc.$pmc_pcerx.write_with_zero(|w| w.$pid().set_bit());
                    let variant = self.get_mode(&config.channel_mode);
                    let parity = self.get_parity(&config.parity);
                    uart.mr.write_with_zero(|w|
                        w.chmode().variant(variant)
                         .par().variant(parity)
                         .filter().bit(config.digital_filter)
                    );

                    let read_baud_rate = 12_000_000u32 / ((config.baud_rate.0 as u32) * 16u32);
                    uart.brgr.write_with_zero(|w| unsafe { w.bits(read_baud_rate) });
                }
            }
        )+
    }
}

uart! {
    UART0: (uart0, uart0tx, uart0rx, pmc_pcer0, pid7),
    UART1: (uart1, uart1tx, uart1rx, pmc_pcer0, pid8),
    UART2: (uart2, uart2tx, uart2rx, pmc_pcer1, pid44),
    UART3: (uart3, uart3tx, uart3rx, pmc_pcer1, pid45),
    UART4: (uart4, uart4tx, uart4rx, pmc_pcer1, pid46),
}
