#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::unix_socket::UnixSocketTransport as NFC;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::nfc::NfcTransport as NFC;

