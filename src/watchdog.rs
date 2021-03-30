use crate::pac::WDT;
use embedded_hal::watchdog;

pub struct Watchdog {
    wdt: WDT,
}

impl Watchdog {
    pub fn new(wdt: WDT) -> Self {
        Self { wdt }
    }
}

impl watchdog::WatchdogEnable for Watchdog {
    type Time = u16;
    fn start<T>(&mut self, period: T)
    where
        T: Into<Self::Time>
    {
        const WATCHDOG_VALUE_MASK: u16 = 0x0FFF;
        let period = period.into() & WATCHDOG_VALUE_MASK;
        let mr = &self.wdt.mr;
        mr.write(|w| {
            w.wddis().clear_bit();
            unsafe { w.wdv().bits(period) }
        });
    }
}

impl watchdog::WatchdogDisable for Watchdog {
    fn disable(&mut self) {
        let mr = &self.wdt.mr;
        mr.write(|w| w.wddis().set_bit())
    }
}

impl watchdog::Watchdog for Watchdog {
    fn feed(&mut self) {
        let cr = &self.wdt.cr;
        cr.write_with_zero(|w|
            w.key().passwd()
        );
    }
}
