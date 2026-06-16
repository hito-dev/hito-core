use crate::drivers::payload::{PayloadBuffer, Error};

static HEX: [u8; 16] = *b"0123456789abcdef";

// Transport trait for communication with external world, e.g. Bluetooth, USB, NFC, QR
pub trait Transport {

    fn init(&mut self) -> bool; 
    fn poll_rx(&mut self) -> Result<(), Error>;

    fn has_data(&self) -> bool;
    fn payload(&self)  -> &[u8];
    fn payload_mut(&mut self) -> &mut [u8];

    fn consume_payload(&mut self, n: usize);
    fn clear_payload(&mut self);

    fn send(&mut self, data: &[u8]) -> bool;

    fn send_hex(&mut self, data: &[u8]) -> bool {

        let mut buf = [0u8; 512];

        let mut sent = 0;

        while sent < data.len() {

            let mut prefix = 0;

            if sent == 0 {
                buf[0] = b'0';
                buf[1] = b'x';
                prefix += 2;
            }

            let chunk_size = core::cmp::min((buf.len() - prefix) / 2, data.len() - sent);

            let chunk = &data[sent..sent + chunk_size];

            for (i, byte) in chunk.iter().enumerate() {
                buf[prefix + i * 2] = HEX[(byte >> 4) as usize];
                buf[prefix + i * 2 + 1] = HEX[(byte & 0x0F) as usize];
            }

            self.send(&buf[..prefix + chunk_size * 2]);

            sent += chunk_size;
        }

        return true;
    }

}

// TransportDriver trait for implementing specific transport drivers for specific platforms, e.g. Bluetooth, USB, NFC, QR
pub trait TransportDriver {
    fn init<B>(&mut self, payload: &mut PayloadBuffer<B>) -> bool
    where 
        B: AsRef<[u8]> + AsMut<[u8]>;

    fn poll_rx<B>(&mut self, payload: &mut PayloadBuffer<B>) -> Result<(), Error>
    where
        B: AsRef<[u8]> + AsMut<[u8]>;

    fn send(&mut self, data: &[u8]) -> bool;
}

// BufferedTransport owns a payload buffer and a specific transport
pub struct TransportDevice<D, B>
where 
    D: TransportDriver,
    B: AsRef<[u8]> + AsMut<[u8]>,
{
    driver: D,
    payload: PayloadBuffer<B>,
    initialized: bool,
}

impl<D, B> TransportDevice<D, B>
where 
    D: TransportDriver,
    B: AsRef<[u8]> + AsMut<[u8]>,
{
    pub fn new(driver: D, storage: B) -> Self {
        Self {
            driver,
            payload: PayloadBuffer::new(storage),
            initialized: false,
        }
    }
}

impl<D, B> Transport for TransportDevice<D, B>
where 
    D: TransportDriver,
    B: AsRef<[u8]> + AsMut<[u8]>,
{
    fn init(&mut self) -> bool {
        self.initialized = self.driver.init(&mut self.payload);
        self.initialized
    }

    fn poll_rx(&mut self) -> Result<(), Error> {
        if !self.initialized {
            return Err(Error::NotInitialized)
        }
        self.driver.poll_rx(&mut self.payload)
    }

    fn has_data(&self) -> bool {
        !self.payload.is_empty()
    }

    fn payload(&self) -> &[u8] {
        self.payload.as_slice()
    }

    fn payload_mut(&mut self) -> &mut [u8] {
        self.payload.storage_mut()
    }

    fn consume_payload(&mut self, n: usize) {
        self.payload.consume(n);
    }

    fn clear_payload(&mut self) {
        self.payload.clear();
    }

    fn send(&mut self, data: &[u8]) -> bool {
        self.initialized && self.driver.send(data)
    }

    /*
    fn lock_payload(&mut self) {
        self.payload.lock();
    }

    fn release_payload(&mut self) -> bool {
        if self.payload.is_locked() {
            self.payload.unlock();
            true
        } else {
            false
        }
    }

    fn has_line(&mut self) -> bool {
        let payload = self.payload();
        if payload.is_empty() {
            return false;
        }
        payload.iter().any(|&b| b == b'\n' || b == b'\r')
    }

    fn get_line(&mut self) -> Option<&str> {
        let payload = self.payload();
        if payload.is_empty() {
            return None;
        }
        if let Some(pos) = payload.iter().position(|&b| b == b'\n' || b == b'\r') {
            let line_bytes = &payload[..pos];
            str::from_utf8(line_bytes).ok()
        } else {
            None
        }
    }

    fn consume_line(&mut self) {
        let payload = self.payload();
        if let Some(pos) = payload.iter().position(|&b| b == b'\n' || b == b'\r') {
            self.consume_payload(pos + 1);
        } else {
            self.consume_payload(payload.len());
        }
    }
    */

}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    struct FakeDriver {
        init_called: bool,
        send_data: Vec<u8>,
        rx_chunks: Vec<Vec<u8>>,
    }

    impl FakeDriver {
        fn new() -> Self {
            Self {
                init_called: false,
                send_data: Vec::new(),
                rx_chunks: Vec::new(),
            }
        }

        fn with_rx(chunks: &[&[u8]]) -> Self {
            Self {
                init_called: false,
                send_data: Vec::new(),
                rx_chunks: chunks.iter().map(|c| c.to_vec()).collect(),
            }
        }
    }

    impl TransportDriver for FakeDriver {
        fn init<B>(&mut self, _payload: &mut PayloadBuffer<B>) -> bool
        where
            B: AsRef<[u8]> + AsMut<[u8]>,
        {
            self.init_called = true;
            true
        }

        fn poll_rx<B>(&mut self, payload: &mut PayloadBuffer<B>) -> Result<(), Error>
        where
            B: AsRef<[u8]> + AsMut<[u8]>,
        {
            if self.rx_chunks.is_empty() {
                return Ok(());
            }

            let chunk = self.rx_chunks.remove(0);
            payload.push(&chunk)
        }

        fn send(&mut self, data: &[u8]) -> bool {
            self.send_data.extend_from_slice(data);
            true
        }
    }

    #[test]
    fn transport_device_init_calls_driver() {
        let driver = FakeDriver::new();
        let mut transport = TransportDevice::new(driver, [0u8; 128]);

        assert!(transport.init());
        assert!(transport.driver.init_called);
    }

    #[test]
    fn transport_device_poll_rx_fills_payload() {
        let driver = FakeDriver::with_rx(&[b"abc"]);
        let mut transport = TransportDevice::new(driver, [0u8; 128]);

        transport.init();
        transport.poll_rx().unwrap();

        assert!(transport.has_data());
        assert_eq!(transport.payload(), b"abc");
    }

    #[test]
    fn transport_device_poll_rx_appends_chunks() {
        let driver = FakeDriver::with_rx(&[b"abc", b"def"]);
        let mut transport = TransportDevice::new(driver, [0u8; 128]);

        transport.init();
        transport.poll_rx().unwrap();
        transport.poll_rx().unwrap();

        assert_eq!(transport.payload(), b"abcdef");
    }

    #[test]
    fn transport_device_consume_payload_keeps_remaining() {
        let driver = FakeDriver::with_rx(&[b"abcdef"]);
        let mut transport = TransportDevice::new(driver, [0u8; 128]);

        transport.init();
        transport.poll_rx().unwrap();

        transport.consume_payload(3);

        assert_eq!(transport.payload(), b"def");
    }

    #[test]
    fn transport_device_clear_payload_clears_buffer() {
        let driver = FakeDriver::with_rx(&[b"abc"]);
        let mut transport = TransportDevice::new(driver, [0u8; 128]);

        transport.init();
        transport.poll_rx().unwrap();

        transport.clear_payload();

        assert!(!transport.has_data());
        assert_eq!(transport.payload(), b"");
    }

    #[test]
    fn transport_device_poll_rx_propagates_overflow() {
        let driver = FakeDriver::with_rx(&[b"abcdef"]);
        let mut transport = TransportDevice::new(driver, [0u8; 4]);

        transport.init();

        assert_eq!(
            transport.poll_rx(),
            Err(Error::Overflow)
        );
    }

    #[test]
    fn transport_device_send_delegates_to_driver() {
        let driver = FakeDriver::new();
        let mut transport = TransportDevice::new(driver, [0u8; 128]);

        transport.init();

        assert!(transport.send(b"ok"));
        assert_eq!(transport.driver.send_data, b"ok");
    }

    #[test]
    fn transport_device_poll_rx_before_init_returns_not_initialized() {
        let driver = FakeDriver::with_rx(&[b"abc"]);
        let mut transport = TransportDevice::new(driver, [0u8; 128]);

        assert_eq!(
            transport.poll_rx(),
            Err(Error::NotInitialized)
        );

        assert!(!transport.has_data());
    }

    #[test]
    fn transport_send_hex_sends_hex_representation() {
        let driver = FakeDriver::new();
        let mut transport = TransportDevice::new(driver, [0u8; 128]);

        transport.init();

        assert!(transport.send_hex(b"\x01\x23\x45\x67\x89\xAB\xCD\xEF"));
        assert_eq!(transport.driver.send_data, b"0x0123456789abcdef");
    }

    #[test]
    fn transport_send_hex_sends_hex_representation_big_buffer() {
        let driver = FakeDriver::new();
        let mut transport = TransportDevice::new(driver, [0u8; 128]);

        transport.init();

        let buf = [0x05u8; 1024];

        let pattern = [0x30u8, 0x35u8]; // "05" in hex
        let buf_hex: Vec<u8> = pattern.iter().copied().cycle().take(2048).collect();
        let buf_hex = [&b"0x"[..], &buf_hex[..]].concat();

        assert!(transport.send_hex(&buf));
        assert_eq!(transport.driver.send_data, buf_hex);
    }
}

