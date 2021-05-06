#![no_std]

pub use atsamv71q21 as pac;
pub mod serial;
pub mod gpio;
pub mod watchdog;
pub mod prelude;