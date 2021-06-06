use core::convert::Infallible;
use core::marker::PhantomData;
use crate::pac::PMC;
use embedded_hal::digital::v2::{toggleable, InputPin, OutputPin, StatefulOutputPin};

pub trait GpioExt {
    type Parts;

    fn split(self, pmc: &PMC) -> Self::Parts;
}

trait GpioRegExt {
    fn is_low(&self, pos: u8) -> bool;
    fn is_set_low(&self, pos: u8) -> bool;
    fn set_high(&self, pos: u8);
    fn set_low(&self, pos: u8);
}

pub struct AF0;
pub struct AF1;
pub struct AF2;
pub struct AF3;

pub struct OpenDrain;
pub struct Floating;
pub struct PullDown;
pub struct PullUp;
pub struct Analog;

pub struct Alternate<AF> {
    _mode: PhantomData<AF>,
}

pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

pub struct Output<MODE> {
    _mode: PhantomData<MODE>
}

pub struct Pin<MODE> {
    i: u8,
    port: *const dyn GpioRegExt,
    _mode: PhantomData<MODE>,
}

unsafe impl<MODE> Sync for Pin<MODE> {}
unsafe impl<MODE> Send for Pin<MODE> {}

impl<MODE> StatefulOutputPin for Pin<Output<MODE>> {
    #[inline(always)]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_set_low(self.i) })
    }
}

impl<MODE> OutputPin for Pin<Output<MODE>> {
    type Error = Infallible;

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe { (*self.port).set_low(self.i) };
        Ok(())
    }

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe { (*self.port).set_high(self.i) };
        Ok(())
    }
}

impl<MODE> toggleable::Default for Pin<Output<MODE>> {}

impl InputPin for Pin<Output<OpenDrain>> {
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_low(self.i) })
    }
}

impl<MODE> InputPin for Pin<Input<MODE>> {
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_low(self.i) })
    }
}

macro_rules! gpio_trait {
    ($gpiox:ident) => {
        impl GpioRegExt for crate::pac::$gpiox::RegisterBlock {
            fn is_low(&self, pos: u8) -> bool {
                self.pdsr.read().bits() & (1 << pos) == 0
            }

            fn is_set_low(&self, pos: u8) -> bool {
                self.odsr.read().bits() & (1 << pos) == 0
            }

            fn set_high(&self, pos: u8) {
                unsafe { self.sodr.write_with_zero(|w| w.bits(1 << pos)); }
            }

            fn set_low(&self, pos: u8) {
                unsafe { self.codr.write_with_zero(|w| w.bits(1 << pos)); }
            }
        }
    }
}

gpio_trait!(pioa);
gpio_trait!(piob);
gpio_trait!(pioc);
gpio_trait!(piod);
gpio_trait!(pioe);



macro_rules! gpio {
    ([$($GPIOX:ident, $gpiox:ident, $iopxenr:ident, $PXx:ident, $pidx:ident => [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
    ]),+]) => {
        $(
            pub mod $gpiox {
                use core::marker::PhantomData;
                use core::convert::Infallible;

                use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, toggleable};
                use crate::pac::{$GPIOX, PMC};
                use cortex_m::interrupt::CriticalSection;

                use super::{
                    Alternate, GpioExt, Input, OpenDrain, Output, Floating, PullUp, PullDown,
                    AF0, AF1, AF2, AF3,
                    Pin, GpioRegExt,
                };

                pub struct Parts {
                    $(
                        pub $pxi: $PXi<$MODE>,
                    )+
                }

                impl GpioExt for $GPIOX {
                    type Parts = Parts;

                    fn split(self, pmc: &PMC) -> Parts {
                        unsafe { pmc.pmc_pcer0.write_with_zero(|w| w.$pidx().set_bit()) };
                        Parts {
                            $(
                                $pxi: $PXi { _mode:PhantomData },
                            )+
                        }
                    }
                }

                const MASK: u32 = 1;
                fn _set_alternate_mode(index: usize, mode: u32) {
                    unsafe {
                        let reg = &(*$GPIOX::ptr());
                        let abcdsr0 = reg.abcdsr.first().unwrap();
                        let abcdsr1 = reg.abcdsr.last().unwrap();

                        let value0 = mode & MASK;
                        let value1 = (mode >> 1) & MASK;

                        abcdsr0.modify(|_, w| w.bits(value0 << index));
                        abcdsr1.modify(|_, w| w.bits(value1 << index));
                    }
                }

                $(
                    pub struct $PXi<MODE> {
                        _mode: PhantomData<MODE>,
                    }

                    impl<MODE> $PXi<MODE> {
                        pub fn enable(self, _cs: &CriticalSection) -> Self {
                            unsafe { (*$GPIOX::ptr()).per.write_with_zero(|w| w.bits(1 << $i)) };
                            self
                        }

                        pub fn disable(self, _cs: &CriticalSection) -> Self {
                            unsafe { (*$GPIOX::ptr()).pdr.write_with_zero(|w| w.bits(1 << $i)) };
                            self
                        }

                        pub fn into_alternate_af0(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF0>> {
                            _set_alternate_mode($i, 0);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af1(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF1>> {
                            _set_alternate_mode($i, 1);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af2(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF2>> {
                            _set_alternate_mode($i, 2);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af3(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF3>> {
                            _set_alternate_mode($i, 3);
                            $PXi { _mode: PhantomData }
                        }

                        //TODO: FALTA FLOATING INPUT

                        pub fn into_pull_down_input(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Input<PullDown>> {
                            unsafe { (*$GPIOX::ptr()).ppder.write_with_zero(|w| w.bits(1 << $i)) };
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_pull_up_input(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Input<PullUp>> {
                            unsafe { (*$GPIOX::ptr()).puer.write_with_zero(|w| w.bits(1 << $i)) };
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_output(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Output<OpenDrain>> {
                            unsafe { (*$GPIOX::ptr()).oer.write_with_zero(|w| w.bits(1 << $i)) }
                            $PXi { _mode: PhantomData }
                        }

                        //TODO: FALTA ANALOG
                        //TODO: FALTA PUSH PULL OUTPUT
                        //TODO: FALTA PUSH PULL OUTPUT HS
                    }

                    //TODO: FALTA INTERNAL PULL UP
                    //TODO: FALTA INTERNAL PULL UP
                    //TODO: FALTA INTERNAL OPEN DRAIN

                    impl<MODE> $PXi<Output<MODE>> {
                        pub fn downgrade(self) -> Pin<Output<MODE>> {
                            Pin {
                                i: $i,
                                port: $GPIOX::ptr() as *const dyn GpioRegExt,
                                _mode: self._mode
                            }
                        }
                    }

                    impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                        fn is_set_high(&self) -> Result<bool, Self::Error> {
                            self.is_set_low().map(|v| !v)
                        }

                        fn is_set_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).is_set_low($i)})
                        }
                    }

                    impl<MODE> OutputPin for $PXi<Output<MODE>> {
                        type Error = Infallible;

                        fn set_high(&mut self) -> Result<(), Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).set_high($i) })
                        }

                        fn set_low(&mut self) -> Result<(), Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).set_low($i) })
                        }
                    }

                    impl<MODE> toggleable::Default for $PXi<Output<MODE>> {}

                    impl InputPin for $PXi<Output<OpenDrain>> {
                        type Error = Infallible;

                        fn is_high(&self) -> Result<bool, Self::Error> {
                            self.is_low().map(|v| !v)
                        }

                        fn is_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).is_low($i) })
                        }
                    }

                    impl<MODE> $PXi<Input<MODE>> {
                        pub fn downgrade(self) -> Pin<Input<MODE>> {
                            Pin {
                                i: $i,
                                port: $GPIOX::ptr() as *const dyn GpioRegExt,
                                _mode: self._mode
                            }
                        }
                    }

                    impl<MODE> InputPin for $PXi<Input<MODE>> {
                        type Error = Infallible;

                        fn is_high(&self) -> Result<bool, Self::Error> {
                            self.is_low().map(|v| !v)
                        }

                        fn is_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).is_low($i) })
                        }
                    }
                )+
            }
        )+
    }
}

gpio!([
    PIOA, pioa, iopaen, PA, pid10 => [
        PA0: (pa0, 0, Input<Floating>),
        PA1: (pa1, 1, Input<Floating>),
        PA2: (pa2, 2, Input<Floating>),
        PA3: (pa3, 3, Input<Floating>),
        PA4: (pa4, 4, Input<Floating>),
        PA5: (pa5, 5, Input<Floating>),
        PA6: (pa6, 6, Input<Floating>),
        PA7: (pa7, 7, Input<Floating>),
        PA8: (pa8, 8, Input<Floating>),
        PA9: (pa9, 9, Input<Floating>),
        PA10: (pa10, 10, Input<Floating>),
        PA11: (pa11, 11, Input<Floating>),
        PA12: (pa12, 12, Input<Floating>),
        PA13: (pa13, 13, Input<Floating>),
        PA14: (pa14, 14, Input<Floating>),
        PA15: (pa15, 15, Input<Floating>),
        PA16: (pa16, 16, Input<Floating>),
        PA17: (pa17, 17, Input<Floating>),
        PA18: (pa18, 18, Input<Floating>),
        PA19: (pa19, 19, Input<Floating>),
        PA20: (pa20, 20, Input<Floating>),
        PA21: (pa21, 21, Input<Floating>),
        PA22: (pa22, 22, Input<Floating>),
        PA23: (pa23, 23, Input<Floating>),
        PA24: (pa24, 24, Input<Floating>),
        PA25: (pa25, 25, Input<Floating>),
        PA26: (pa26, 26, Input<Floating>),
        PA27: (pa27, 27, Input<Floating>),
        PA28: (pa28, 28, Input<Floating>),
        PA29: (pa29, 29, Input<Floating>),
        PA30: (pa30, 30, Input<Floating>),
        PA31: (pa31, 31, Input<Floating>),
],
    PIOB, piob, iopben, PB, pid11 => [
        PB0: (pb0, 0, Input<Floating>),
        PB1: (pb1, 1, Input<Floating>),
        PB2: (pb2, 2, Input<Floating>),
        PB3: (pb3, 3, Input<Floating>),
        PB4: (pb4, 4, Input<Floating>),
        PB5: (pb5, 5, Input<Floating>),
        PB6: (pb6, 6, Input<Floating>),
        PB7: (pb7, 7, Input<Floating>),
        PB8: (pb8, 8, Input<Floating>),
        PB9: (pb9, 9, Input<Floating>),
        PB10: (pb10, 10, Input<Floating>),
        PB11: (pb11, 11, Input<Floating>),
        PB12: (pb12, 12, Input<Floating>),
        PB13: (pb13, 13, Input<Floating>),
        PB14: (pb14, 14, Input<Floating>),
        PB15: (pb15, 15, Input<Floating>),
        PB16: (pb16, 16, Input<Floating>),
        PB17: (pb17, 17, Input<Floating>),
        PB18: (pb18, 18, Input<Floating>),
        PB19: (pb19, 19, Input<Floating>),
        PB20: (pb20, 20, Input<Floating>),
        PB21: (pb21, 21, Input<Floating>),
        PB22: (pb22, 22, Input<Floating>),
        PB23: (pb23, 23, Input<Floating>),
        PB24: (pb24, 24, Input<Floating>),
        PB25: (pb25, 25, Input<Floating>),
        PB26: (pb26, 26, Input<Floating>),
        PB27: (pb27, 27, Input<Floating>),
        PB28: (pb28, 28, Input<Floating>),
        PB29: (pb29, 29, Input<Floating>),
        PB30: (pb30, 30, Input<Floating>),
        PB31: (pb31, 31, Input<Floating>),
],
    PIOC, pioc, iopcen, PC, pid12 => [
        PC0: (pc0, 0, Input<Floating>),
        PC1: (pc1, 1, Input<Floating>),
        PC2: (pc2, 2, Input<Floating>),
        PC3: (pc3, 3, Input<Floating>),
        PC4: (pc4, 4, Input<Floating>),
        PC5: (pc5, 5, Input<Floating>),
        PC6: (pc6, 6, Input<Floating>),
        PC7: (pc7, 7, Input<Floating>),
        PC8: (pc8, 8, Input<Floating>),
        PC9: (pc9, 9, Input<Floating>),
        PC10: (pc10, 10, Input<Floating>),
        PC11: (pc11, 11, Input<Floating>),
        PC12: (pc12, 12, Input<Floating>),
        PC13: (pc13, 13, Input<Floating>),
        PC14: (pc14, 14, Input<Floating>),
        PC15: (pc15, 15, Input<Floating>),
        PC16: (pc16, 16, Input<Floating>),
        PC17: (pc17, 17, Input<Floating>),
        PC18: (pc18, 18, Input<Floating>),
        PC19: (pc19, 19, Input<Floating>),
        PC20: (pc20, 20, Input<Floating>),
        PC21: (pc21, 21, Input<Floating>),
        PC22: (pc22, 22, Input<Floating>),
        PC23: (pc23, 23, Input<Floating>),
        PC24: (pc24, 24, Input<Floating>),
        PC25: (pc25, 25, Input<Floating>),
        PC26: (pc26, 26, Input<Floating>),
        PC27: (pc27, 27, Input<Floating>),
        PC28: (pc28, 28, Input<Floating>),
        PC29: (pc29, 29, Input<Floating>),
        PC30: (pc30, 30, Input<Floating>),
        PC31: (pc31, 31, Input<Floating>),
],
    PIOD, piod, iopden, PD, pid16 => [
        PD0: (pd0, 0, Input<Floating>),
        PD1: (pd1, 1, Input<Floating>),
        PD2: (pd2, 2, Input<Floating>),
        PD3: (pd3, 3, Input<Floating>),
        PD4: (pd4, 4, Input<Floating>),
        PD5: (pd5, 5, Input<Floating>),
        PD6: (pd6, 6, Input<Floating>),
        PD7: (pd7, 7, Input<Floating>),
        PD8: (pd8, 8, Input<Floating>),
        PD9: (pd9, 9, Input<Floating>),
        PD10: (pd10, 10, Input<Floating>),
        PD11: (pd11, 11, Input<Floating>),
        PD12: (pd12, 12, Input<Floating>),
        PD13: (pd13, 13, Input<Floating>),
        PD14: (pd14, 14, Input<Floating>),
        PD15: (pd15, 15, Input<Floating>),
        PD16: (pd16, 16, Input<Floating>),
        PD17: (pd17, 17, Input<Floating>),
        PD18: (pd18, 18, Input<Floating>),
        PD19: (pd19, 19, Input<Floating>),
        PD20: (pd20, 20, Input<Floating>),
        PD21: (pd21, 21, Input<Floating>),
        PD22: (pd22, 22, Input<Floating>),
        PD23: (pd23, 23, Input<Floating>),
        PD24: (pd24, 24, Input<Floating>),
        PD25: (pd25, 25, Input<Floating>),
        PD26: (pd26, 26, Input<Floating>),
        PD27: (pd27, 27, Input<Floating>),
        PD28: (pd28, 28, Input<Floating>),
        PD29: (pd29, 29, Input<Floating>),
        PD30: (pd30, 30, Input<Floating>),
        PD31: (pd31, 31, Input<Floating>),
],
    PIOE, pioe, iopeen, PE, pid17 => [
        PE0: (pe0, 0, Input<Floating>),
        PE1: (pe1, 1, Input<Floating>),
        PE2: (pe2, 2, Input<Floating>),
        PE3: (pe3, 3, Input<Floating>),
        PE4: (pe4, 4, Input<Floating>),
        PE5: (pe5, 5, Input<Floating>),
        PE6: (pe6, 6, Input<Floating>),
        PE7: (pe7, 7, Input<Floating>),
        PE8: (pe8, 8, Input<Floating>),
        PE9: (pe9, 9, Input<Floating>),
        PE10: (pe10, 10, Input<Floating>),
        PE11: (pe11, 11, Input<Floating>),
        PE12: (pe12, 12, Input<Floating>),
        PE13: (pe13, 13, Input<Floating>),
        PE14: (pe14, 14, Input<Floating>),
        PE15: (pe15, 15, Input<Floating>),
        PE16: (pe16, 16, Input<Floating>),
        PE17: (pe17, 17, Input<Floating>),
        PE18: (pe18, 18, Input<Floating>),
        PE19: (pe19, 19, Input<Floating>),
        PE20: (pe20, 20, Input<Floating>),
        PE21: (pe21, 21, Input<Floating>),
        PE22: (pe22, 22, Input<Floating>),
        PE23: (pe23, 23, Input<Floating>),
        PE24: (pe24, 24, Input<Floating>),
        PE25: (pe25, 25, Input<Floating>),
        PE26: (pe26, 26, Input<Floating>),
        PE27: (pe27, 27, Input<Floating>),
        PE28: (pe28, 28, Input<Floating>),
        PE29: (pe29, 29, Input<Floating>),
        PE30: (pe30, 30, Input<Floating>),
        PE31: (pe31, 31, Input<Floating>),
]]);
