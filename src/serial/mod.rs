pub mod uart;
pub mod usart;

pub struct BaudRate(pub u32);

impl Into<BaudRate> for u32 {
    fn into(self) -> BaudRate {
        BaudRate(self)
    }
}
