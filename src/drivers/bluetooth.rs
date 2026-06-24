#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::unix_socket::UnixSocketTransport as Bluetooth;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::bluetooth::BluetoothZephyrTransport as Bluetooth;

