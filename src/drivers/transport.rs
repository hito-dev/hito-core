use crate::drivers::payload::{PayloadBuffer, PayloadError};

// Transport trait for communication with external world, e.g. Bluetooth, USB, NFC, QR
pub trait Transport {

    fn init(&mut self) -> bool; 
    fn poll_rx(&mut self) -> Result<(), PayloadError>;

    fn has_data(&self) -> bool;
    fn payload(&self)  -> &[u8];

    fn consume_payload(&mut self, n: usize);
    fn clear_payload(&mut self);

    fn send(&mut self, data: &[u8]) -> bool;
}

// TransportDriver trait for implementing specific transport drivers for specific platforms, e.g. Bluetooth, USB, NFC, QR
pub trait TransportDriver {
    fn init<B>(&mut self, payload: &mut PayloadBuffer<B>) -> bool
    where 
        B: AsRef<[u8]> + AsMut<[u8]>;

    fn poll_rx<B>(&mut self, payload: &mut PayloadBuffer<B>) -> Result<(), PayloadError>
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

    fn poll_rx(&mut self) -> Result<(), PayloadError> {
        if !self.initialized {
            return Err(PayloadError::NotInitialized)
        }
        self.driver.poll_rx(&mut self.payload)
    }

    fn has_data(&self) -> bool {
        !self.payload.is_empty()
    }

    fn payload(&self) -> &[u8] {
        self.payload.as_slice()
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

        fn poll_rx<B>(&mut self, payload: &mut PayloadBuffer<B>) -> Result<(), PayloadError>
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
            Err(PayloadError::Overflow)
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
            Err(PayloadError::NotInitialized)
        );

        assert!(!transport.has_data());
    }
}

