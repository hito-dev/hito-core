#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::unix_socket::UnixSocketTransport as UsbSerial;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::serial::UsbSerialTransport as UsbSerial;
// TODO implement UsbZerpyrTransport instead 

