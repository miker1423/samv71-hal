#![no_std]

pub use atsamv71q21 as pac;
pub mod uart;
pub mod gpio;
pub mod prelude;