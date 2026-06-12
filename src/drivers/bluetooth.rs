use crate::driver;

// TODO implement ping pong by default in debug mode
// TODO implement a bluetooth transport for zephyr

#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::unix_socket::UnixSocketTransport as Bluetooth;

//#[cfg(feature = "zephyr")]
//pub use crate::drivers::zephyr::bluetooth::BluetoothZephyr as Bluetooth;

