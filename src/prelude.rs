pub use embedded_hal::prelude::*;

pub use embedded_hal::digital::v2::InputPin as _embedded_hal_gpio_InputPin;
pub use embedded_hal::digital::v2::OutputPin as _embedded_hal_gpio_OutputPin;
pub use embedded_hal::digital::v2::StatefulOutputPin as _embedded_hal_gpio_StatefulOutputPin;
pub use embedded_hal::digital::v2::ToggleableOutputPin as _embedded_hal_gpio_ToggleableOutputPin;

pub use crate::gpio::GpioExt as _samv71q_hal_gpio_GpioExt;
