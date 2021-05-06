pub mod uart;
pub mod usart;

pub struct BaudRate(pub u16);

impl Into<BaudRate> for u16 {
    fn into(self) -> BaudRate {
        BaudRate(self)
    }
}
