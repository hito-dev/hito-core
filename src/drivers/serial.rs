use crate::driver;
use alloc::string::String;

driver! {
    pub trait SerialDriver => UsbSerial {
        fn init() -> bool;

        fn has_data() -> bool;
        fn get_data_len() -> usize;
        fn get_data(out: &mut [u8]) -> usize;
        fn clear_data();

        fn has_line() -> bool;

        // TODO modify to str
        fn get_line() -> Option<String>;

        fn send(data: &[u8]) -> bool;
    }
}

#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::serial::SerialDesktop as UsbSerial;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::serial::SerialZephyr as UsbSerial;

