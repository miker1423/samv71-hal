use embedded_hal::serial::{Read, Write};
use core::{convert::Infallible, marker::PhantomData };
use crate::{gpio::*, serial::BaudRate, pac::PMC};

pub enum Parity {
    Even,
    Odd,
    Space,
    Mark,
    NoParity,
    MultridropMode
}

pub enum ChannelMode {
    Normal,
    Automatic,
    LocalLoopback,
    RemoteLoopback
}

#[derive(PartialOrd, PartialEq)]
pub enum SyncMode {
    Async,
    Sync
}

pub enum CharLength {
    FiveBit,
    SixBit,
    SevenBit,
    EightBit
}

pub enum UsartMode {
    Normal,
    Rs485,
    HWHandsahking,
    LON,
    SPIMaster,
    SPISlave
}

pub enum UsartError {
    Parity,
    Framing,
    Overrun
}

pub trait RxPin<USART> {}
pub trait TxPin<USART> {}

macro_rules! usart_pins {
    ($($USART:ident => {
        tx => [$($tx:ty), +$(,)*],
        rx => [$($rx:ty), +$(,)*],
    })+) => {
        $(
            $(
                impl TxPin<crate::pac::$USART> for $tx {}
            )+
            $(
                impl RxPin<crate::pac::$USART> for $rx {}
            )+
        )+
    }
}

pub struct Rx<USART> {
    _instance: PhantomData<USART>,
}

pub struct Tx<USART> {
    _instance: PhantomData<USART>,
}

pub struct Serial<USART, TXPIN, RXPIN> {
    usart: USART,
    pins: (TXPIN, RXPIN),
}

impl<USART, TXPIN, RXPIN> Serial<USART, TXPIN, RXPIN> {
    pub fn split(self) -> (Tx<USART>, Rx<USART>)
        where
            TXPIN: TxPin<USART>,
            RXPIN: RxPin<USART>,
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

    pub fn release(self) -> (TXPIN, RXPIN) { self.pins }
}

usart_pins! {
    USART0 => {
        tx => [piob::PB1<Alternate<AF2>>],
        rx => [piob::PB0<Alternate<AF2>>],
    }
    USART1 => {
        tx => [piob::PB4<Alternate<AF3>>],
        rx => [pioa::PA21<Alternate<AF0>>],
    }
    USART2 => {
        tx => [piod::PD16<Alternate<AF1>>],
        rx => [piod::PD15<Alternate<AF1>>],
    }
}

pub struct Config {
    baud_rate: BaudRate,
    parity: Parity,
    channel_mode: ChannelMode,
    char_length: CharLength,
    sync_mode: SyncMode,
    usart_mode: UsartMode,
}

impl Config {
    pub fn new(
        baud_rate: BaudRate,
        parity: Parity,
        channel_mode: ChannelMode,
        char_length: CharLength,
        sync_mode: SyncMode,
        usart_mode: UsartMode) -> Config {
        Config {
            baud_rate,
            parity,
            channel_mode,
            char_length,
            sync_mode,
            usart_mode,
        }
    }
}

trait ConfigMethod {
    type Parity;
    type Mode;
    type CharLength;
    type UsartMode;

    fn get_parity(config: &Config) -> Self::Parity;

    fn get_mode(config: &Config) -> Self::Mode;

    fn get_char_length(config: &Config) -> Self::CharLength;

    fn get_usart_mode(config: &Config) -> Self::UsartMode;

    fn configure(&self, config: &Config, pmc: &PMC);
}

macro_rules! usart {
    ($($USART:ident: ($usart:ident, $usarttx:ident, $usartrx:ident, $pmc_pcerx:ident, $pid:ident),)+) => {
        $(
            use crate::pac::$USART;

            impl<TXPIN, RXPIN> Serial<$USART, TXPIN, RXPIN>
            where
                TXPIN: TxPin<$USART>,
                RXPIN: RxPin<$USART>,
            {
                pub fn $usart(usart: $USART, pins: (TXPIN, RXPIN), config: &Config, pmc: &PMC) -> Self {
                    let serial = Serial { usart, pins };
                    serial.configure(config, pmc);
                    serial.usart.cr().write_with_zero(|w| w.txen().set_bit().rxen().set_bit());
                    serial
                }
            }

            impl<TXPIN> Serial<$USART, TXPIN, ()>
                where
                    TXPIN: TxPin<$USART>,
            {
                pub fn $usarttx(usart: $USART, txpin: TXPIN, config: &Config, pmc: &PMC) -> Self {
                    let rxpin = ();
                    let serial = Serial { usart, pins: (txpin, rxpin) };
                    serial.configure(config, pmc);
                    serial.usart.cr().write_with_zero(|w| w.txen().set_bit());
                    serial
                }
            }

            impl<RXPIN> Serial<$USART, (), RXPIN>
                where
                    RXPIN: RxPin<$USART>
            {
                pub fn $usartrx(usart: $USART, rxpin: RXPIN, config: &Config, pmc: &PMC) -> Self {
                    let txpin = ();
                    let serial = Serial { usart, pins: (txpin, rxpin) };
                    serial.configure(config, pmc);
                    serial.usart.cr().write_with_zero(|w| w.rxen().set_bit());
                    serial
                }
            }


            impl core::fmt::Write for Tx<$USART>
                where
                    Tx<$USART>: embedded_hal::serial::Write<u16>
            {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    s.as_bytes()
                        .iter()
                        .try_for_each(|c| nb::block!(self.write((*c).into())))
                        .map_err(|_| core::fmt::Error)
                }
            }

            impl<TXPIN, RXPIN> core::fmt::Write for Serial<$USART, TXPIN, RXPIN>
                where
                    TXPIN: TxPin<$USART>
            {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    s.as_bytes()
                        .iter()
                        .try_for_each(|c| nb::block!(self.write((*c).into())))
                        .map_err(|_| core::fmt::Error)
                }
            }

            impl<TXPIN, RXPIN> Read<u16> for Serial<$USART, TXPIN, RXPIN>
                where
                    RXPIN: RxPin<$USART>
            {
                type Error = UsartError;

                fn read(&mut self) -> nb::Result<u16, Self::Error>
                {
                    let status_register = unsafe { (&*$USART::ptr()).csr().read() };
                    if status_register.ovre().bit() {
                        Err(nb::Error::Other(UsartError::Overrun))
                    } else if status_register.frame().bit() {
                        Err(nb::Error::Other(UsartError::Framing))
                    } else if status_register.pare().bit() {
                        Err(nb::Error::Other(UsartError::Parity))
                    } else if status_register.rxrdy().bit() {
                        let rhr = unsafe { (&*$USART::ptr()).rhr.read() };
                        let value = rhr.rxchr().bits();
                        Ok(value)
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl Read<u16> for Rx<$USART>
            {
                type Error = UsartError;

                fn read(&mut self) -> nb::Result<u16, Self::Error>
                {
                    let status_register = unsafe { (&*$USART::ptr()).csr().read() };
                    if status_register.ovre().bit() {
                        Err(nb::Error::Other(UsartError::Overrun))
                    } else if status_register.frame().bit() {
                        Err(nb::Error::Other(UsartError::Framing))
                    } else if status_register.pare().bit() {
                        Err(nb::Error::Other(UsartError::Parity))
                    } else if status_register.rxrdy().bit() {
                        let rhr = unsafe { (&*$USART::ptr()).rhr.read() };
                        let value = rhr.rxchr().bits();
                        Ok(value)
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl<TXPIN, RXPIN> Write<u16> for Serial<$USART, TXPIN, RXPIN>
                where
                    TXPIN: TxPin<$USART>
            {
                type Error = Infallible;

                fn write(&mut self, data: u16) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$USART::ptr()).csr().read() };
                    if status_register.txrdy().bit() {
                        let usart = unsafe { (&*$USART::ptr())};
                        usart.thr.write_with_zero(|w| unsafe { w.txchr().bits(data) });
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                fn flush(&mut self) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$USART::ptr()).csr().read() };
                    if status_register.txempty().bit() {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl Write<u16> for Tx<$USART>
            {
                type Error = Infallible;

                fn write(&mut self, data: u16) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$USART::ptr()).csr().read() };
                    if status_register.txrdy().bit() {
                        let usart = unsafe { (&*$USART::ptr())};
                        usart.thr.write_with_zero(|w| unsafe { w.txchr().bits(data) });
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                fn flush(&mut self) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$USART::ptr()).csr().read() };
                    if status_register.txempty().bit() {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl<TXPIN, RXPIN> ConfigMethod for Serial<$USART, TXPIN, RXPIN> {
                type Parity = crate::pac::$usart::mr::PAR_A;
                type Mode = crate::pac::$usart::mr::CHMODE_A;
                type CharLength = crate::pac::$usart::mr::CHRL_A;
                type UsartMode = crate::pac::$usart::mr::USART_MODE_A;

                fn get_parity(config: &Config) -> Self::Parity {
                    match config.parity {
                        Parity::Even => Self::Parity::EVEN,
                        Parity::Mark => Self::Parity::MARK,
                        Parity::MultridropMode => Self::Parity::MULTIDROP,
                        Parity::NoParity => Self::Parity::NO,
                        Parity::Odd => Self::Parity::ODD,
                        Parity::Space => Self::Parity::SPACE,
                    }
                }

                fn get_mode(config: &Config) -> Self::Mode {
                    match config.channel_mode  {
                        ChannelMode::Normal => Self::Mode::NORMAL,
                        ChannelMode::Automatic => Self::Mode::AUTOMATIC,
                        ChannelMode::LocalLoopback => Self::Mode::LOCAL_LOOPBACK,
                        ChannelMode::RemoteLoopback => Self::Mode::REMOTE_LOOPBACK,
                    }
                }

                fn get_char_length(config: &Config) -> Self::CharLength {
                    match config.char_length {
                        CharLength::FiveBit => Self::CharLength::_5_BIT,
                        CharLength::SixBit => Self::CharLength::_6_BIT,
                        CharLength::SevenBit => Self::CharLength::_7_BIT,
                        CharLength::EightBit => Self::CharLength::_8_BIT,
                    }
                }

                fn get_usart_mode(config: &Config) -> Self::UsartMode {
                    match config.usart_mode {
                        UsartMode::Normal => Self::UsartMode::NORMAL,
                        UsartMode::HWHandsahking => Self::UsartMode::HW_HANDSHAKING,
                        UsartMode::LON => Self::UsartMode::LON,
                        UsartMode::Rs485 => Self::UsartMode::RS485,
                        UsartMode::SPISlave => Self::UsartMode::SPI_SLAVE,
                        UsartMode::SPIMaster => Self::UsartMode::SPI_MASTER,
                    }
                }

                fn configure(&self, config: &Config, pmc: &PMC) {
                    let usart = &self.usart;
                    pmc.$pmc_pcerx.write_with_zero(|w| w.$pid().set_bit());
                    let mode = Self::get_mode(config);
                    let parity = Self::get_parity(config);
                    let usart_mode = Self::get_usart_mode(config);
                    let char_length = Self::get_char_length(config);
                    let is_sync = config.sync_mode == SyncMode::Sync;
                    usart.mr().write_with_zero(|w| {
                        w.usart_mode().variant(usart_mode)
                            .par().variant(parity)
                            .chmode().variant(mode)
                            .chrl().variant(char_length)
                            .sync().bit(is_sync)
                    });

                    let read_baud_rate = 12_000_000u32 / ((config.baud_rate.0 as u32) * 16u32);
                    usart.brgr.write_with_zero(|w| unsafe { w.bits(read_baud_rate) });
                }
            }
        )+
    }
}

usart! {
    USART0: (usart0, usart0tx, usart0rx, pmc_pcer0, pid13),
    USART1: (usart1, usart1tx, usart1rx, pmc_pcer0, pid14),
    USART2: (usart2, usart2tx, usart2rx, pmc_pcer0, pid15),
}
