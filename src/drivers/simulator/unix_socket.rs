use std::{
    fs, io, path::PathBuf,
    os::unix::net::{UnixListener, UnixStream},
};
use crate::drivers::{
    payload::{PayloadBuffer, PayloadError},
    transport::{TransportDevice, TransportDriver}
};

pub type UnixSocketTransport<B> = TransportDevice<UnixSocketDriver, B>;

pub struct UnixSocketDriver {
    path     : PathBuf,
    listener : Option<UnixListener>,
    stream   : Option<UnixStream>,
}

impl UnixSocketDriver {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            listener: None,
            stream: None,
        }
    }
}

impl TransportDriver for UnixSocketDriver {
    fn init<B>(&mut self, _payload: &mut PayloadBuffer<B>) -> bool
    where 
        B: AsRef<[u8]> + AsMut<[u8]>,
    {
        let _ = fs::remove_file(&self.path);

        let listener = match UnixListener::bind(&self.path) {
            Ok(listener) => listener,
            Err(_) => return false,
        };

        if listener.set_nonblocking(true).is_err() {
            return false;
        }
        
        self.listener = Some(listener);
        true
    }

    fn poll_rx<B>(&mut self, payload: &mut PayloadBuffer<B>) -> Result<(), PayloadError>
    where
        B: AsRef<[u8]> + AsMut<[u8]>,
    {
        // Accept connection if we don't have one
        if self.stream.is_none() {
            if let Some(listener) = &self.listener {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = stream.set_nonblocking(true);
                        self.stream = Some(stream);
                    }
                    Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                        return Ok(());
                    },
                    Err(_) => { 
                        return Ok(());
                    }
                }
            }
        }

        let Some(stream) = self.stream.as_mut() else {
            return Ok(());
        };

        let mut tmp = [0u8; 256];

        loop {
            match io::Read::read(stream, &mut tmp) {
                Ok(0) => {
                    // Connection closed
                    self.stream = None;
                    break;
                }
                Ok(n) => {
                    trace!("Received {} bytes from Unix socket", n);
                    payload.push(&tmp[..n])?;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => {
                    self.stream = None;
                    break;
                }
            }
        }

        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> bool {
        let Some(stream) = self.stream.as_mut() else {
            return false;
        };

        io::Write::write_all(stream, data).is_ok() && io::Write::flush(stream).is_ok()
    }
}

impl UnixSocketTransport<&'static mut [u8]> {
    pub fn take() -> Option<Self> {
        Some(
            Self::new(
                UnixSocketDriver::new("/tmp/hito.sock"), 
                crate::drivers::payload_storage::take()?,
            )
        )
    }
}

//----------------------------------------------------------------------------------------------
// TESTS
//----------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::transport::Transport;
    use std::{
        vec, format,
        io::{Read, Write},
        os::unix::net::UnixStream,
        path::PathBuf,
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    fn test_socket_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!(
            "hito-{}-{}-{}.sock",
            name,
            std::process::id(),
            now,
        ));

        let _ = std::fs::remove_file(&path);
        path
    }

    fn poll_until_data<T: Transport>(transport: &mut T) {
        for _ in 0..50 {
            transport.poll_rx().unwrap();

            if transport.has_data() {
                return;
            }

            thread::sleep(Duration::from_millis(5));
        }

        panic!("transport did not receive data");
    }

    #[test]
    fn unix_socket_poll_rx_without_client_does_not_block() {
        let path = test_socket_path("no-client");

        let driver = UnixSocketDriver::new(&path);
        let mut transport = UnixSocketTransport::new(driver, vec![0u8; 4096]);

        assert!(transport.init());

        transport.poll_rx().unwrap();

        assert!(!transport.has_data());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unix_socket_poll_rx_receives_payload() {
        let path = test_socket_path("receive");

        let driver = UnixSocketDriver::new(&path);
        let mut transport = UnixSocketTransport::new(driver, vec![0u8; 4096]);

        assert!(transport.init());

        let mut client = UnixStream::connect(&path).unwrap();
        client.write_all(b"hello").unwrap();

        poll_until_data(&mut transport);

        assert_eq!(transport.payload(), b"hello");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unix_socket_consume_payload_keeps_remaining_bytes() {
        let path = test_socket_path("consume");

        let driver = UnixSocketDriver::new(&path);
        let mut transport = UnixSocketTransport::new(driver, vec![0u8; 4096]);

        assert!(transport.init());

        let mut client = UnixStream::connect(&path).unwrap();
        client.write_all(b"abcdef").unwrap();

        poll_until_data(&mut transport);

        transport.consume_payload(3);

        assert_eq!(transport.payload(), b"def");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unix_socket_clear_payload_removes_data() {
        let path = test_socket_path("clear");

        let driver = UnixSocketDriver::new(&path);
        let mut transport = UnixSocketTransport::new(driver, vec![0u8; 4096]);

        assert!(transport.init());

        let mut client = UnixStream::connect(&path).unwrap();
        client.write_all(b"abc").unwrap();

        poll_until_data(&mut transport);

        transport.clear_payload();

        assert!(!transport.has_data());
        assert_eq!(transport.payload(), b"");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unix_socket_send_writes_to_client() {
        let path = test_socket_path("send");

        let driver = UnixSocketDriver::new(&path);
        let mut transport = UnixSocketTransport::new(driver, vec![0u8; 4096]);

        assert!(transport.init());

        let mut client = UnixStream::connect(&path).unwrap();

        // accept client
        transport.poll_rx().unwrap();

        assert!(transport.send(b"ok"));

        let mut buf = [0u8; 16];
        let n = client.read(&mut buf).unwrap();

        assert_eq!(&buf[..n], b"ok");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unix_socket_poll_rx_returns_overflow() {
        let path = test_socket_path("overflow");

        let driver = UnixSocketDriver::new(&path);
        let mut transport = UnixSocketTransport::new(driver, [0u8; 4]);

        assert!(transport.init());

        let mut client = UnixStream::connect(&path).unwrap();
        client.write_all(b"abcdef").unwrap();

        let mut result = Ok(());

        for _ in 0..50 {
            result = transport.poll_rx();

            if result == Err(PayloadError::Overflow) {
                break;
            }

            thread::sleep(Duration::from_millis(5));
        }

        assert_eq!(result, Err(PayloadError::Overflow));

        let _ = std::fs::remove_file(path);
    }
}
