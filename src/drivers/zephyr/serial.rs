// serial.rs
#![allow(dead_code)]

use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use alloc::string::String;

use crate::drivers::Time;
// use crate::drivers::serial::SerialDriver;

use crate::drivers::{
    payload::{PayloadBuffer, Error},
    transport::{TransportDevice, TransportDriver}
};

// ----- Zephyr FFI -----

#[repr(C)]
pub struct device {
    _priv: [u8; 0],
}

extern "C" {
    #[link_name = "hito_platform_device_get_binding"]
    fn device_get_binding(name: *const core::ffi::c_char) -> *const device;

    // In Zephyr, usb_enable takes an optional status callback device pointer in some versions.
    // Commonly used as usb_enable(NULL).
    #[link_name = "hito_platform_usb_enable"]
    fn usb_enable() -> i32;

    #[link_name = "hito_platform_uart_poll_in"]
    fn uart_poll_in(dev: *const device, c: *mut u8) -> i32;
    #[link_name = "hito_platform_uart_poll_out"]
    fn uart_poll_out(dev: *const device, c: u8);

    #[link_name = "hito_platform_uart_line_ctrl_get"]
    fn uart_line_ctrl_get(dev: *const device, ctrl: u32, val: *mut u32) -> i32;
    #[link_name = "hito_platform_uart_line_ctrl_set"]
    fn uart_line_ctrl_set(dev: *const device, ctrl: u32, val: u32) -> i32;
}

// UART line control constants (Zephyr uart_line_ctrl). These values are stable across Zephyr releases.
const UART_LINE_CTRL_DTR: u32 = 1;
const UART_LINE_CTRL_DCD: u32 = 3;
const UART_LINE_CTRL_DSR: u32 = 4;

// ----- Simple lock-free-ish ring buffer (single producer/consumer in one thread) -----
// struct Ring {
//     len: AtomicUsize,
// }
// unsafe impl Sync for Ring {}

// // use crate::drivers::payload_buffer;

// impl Ring {
//     const fn new() -> Self {
//         Self {
//             len: AtomicUsize::new(0),
//         }
//     }

//     #[inline]
//     fn cap(&self) -> usize {
//         crate::drivers::payload_storage::payload_buffer::capacity()
//     }

//     #[inline]
//     fn ptr(&self) -> *mut u8 {
//         crate::drivers::payload_storage::payload_buffer::as_mut_ptr()
//     }

//     #[inline]
//     fn len(&self) -> usize {
//         self.len.load(Ordering::Acquire)
//     }

//     #[inline]
//     fn clear(&self) {
//         self.len.store(0, Ordering::Release);
//     }

//     #[inline]
//     fn push(&self, b: u8) -> bool {
//         let current_len = self.len.load(Ordering::Relaxed);
//         if current_len >= self.cap() {
//             return false;
//         }

//         unsafe {
//             self.ptr().add(current_len).write(b);
//         }
//         self.len.store(current_len + 1, Ordering::Release);
//         true
//     }

//     #[inline]
//     fn read_all(&self, out: &mut [u8]) -> usize {
//         let current_len = self.len.load(Ordering::Acquire);
//         let n = core::cmp::min(current_len, out.len());

//         unsafe {
//             core::ptr::copy_nonoverlapping(
//                 self.ptr() as *const u8,
//                 out.as_mut_ptr(),
//                 n
//             );
//         }
//         n
//     }

//     #[inline]
//     fn has_line(&self) -> bool {
//         let current_len = self.len.load(Ordering::Acquire);
//         if current_len == 0 {
//             return false;
//         }
//         unsafe {
//             self.ptr().add(current_len - 1).read() == b'\n'
//         }
//     }

//     #[inline]
//     fn get_line(&self) -> Option<String> {
//         let current_len = self.len.load(Ordering::Acquire);
//         if current_len == 0 {
//             return None;
//         }

//         unsafe {
//             let bytes = core::slice::from_raw_parts(self.ptr() as *const u8, current_len);
//             // Remove trailing '\n' if present
//             let bytes = if bytes.last() == Some(&b'\n') {
//                 &bytes[..bytes.len() - 1]
//             } else {
//                 bytes
//             };

//             Some(alloc::string::String::from_utf8_lossy(bytes).into_owned())
//         }
//     }

// }

// ----- SerialZephyr implementation -----

// static RX: Ring = Ring::new();

// static READY: AtomicBool = AtomicBool::new(false);
// static CONNECTED: AtomicBool = AtomicBool::new(false);

// // Store pointer as usize to avoid Option<&'static device> tricks in no_std.
// static DEV_PTR: AtomicUsize = AtomicUsize::new(0);

// fn dev() -> *const device {
//     DEV_PTR.load(Ordering::Acquire) as *const device
// }

// fn set_dev(p: *const device) {
//     DEV_PTR.store(p as usize, Ordering::Release);
// }

// fn refresh_connected_flag() {
//     let d = dev();
//     if d.is_null() {
//         CONNECTED.store(false, Ordering::Release);
//         return;
//     }

//     let mut dtr: u32 = 0;
//     let rc = unsafe { uart_line_ctrl_get(d, UART_LINE_CTRL_DTR, &mut dtr as *mut u32) };
//     if rc == 0 && dtr != 0 {
//         CONNECTED.store(true, Ordering::Release);
//     } else {
//         CONNECTED.store(false, Ordering::Release);
//     }
// }

pub struct SerialZephyr;

// fn poll_if_ready() -> bool {
//     if !READY.load(Ordering::Acquire) {
//         return false;
//     }
//     refresh_connected_flag();
//     poll_rx_once();
//     true
// }

pub type UsbSerialTransport<B> = TransportDevice<UsbSerialDriverZephyr, B>;

pub struct UsbSerialDriverZephyr {
    ready: AtomicBool,
    connected: AtomicBool,
    // Store pointer as usize to avoid Option<&'static device> tricks in no_std.
    dev_ptr: AtomicUsize,
    current_len: AtomicUsize,
}

impl UsbSerialDriverZephyr {
    pub fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            connected: AtomicBool::new(false),
            dev_ptr: AtomicUsize::new(0),
            current_len: AtomicUsize::new(0),
        }
    }

    fn push_byte(&self, b: u8, payload: &mut [u8]) -> bool
    {
        //debug!("Received byte: {}", b);
        let current_len = self.current_len.load(Ordering::Relaxed);
        if current_len >= payload.len() {
            debug!("Payload buffer full, dropping byte");
            return false;
        }

        unsafe {
            payload[current_len] = b;
        }
        self.current_len.store(current_len + 1, Ordering::Release);
        true
    }

    fn poll_rx_once(&mut self, payload: &mut [u8]) -> usize {
        let d = self.get_dev();
        if d.is_null() {
            return 0;
        }

        let mut len = 0;

        // Trying to read as much data as possible
        loop {
            let mut c: u8 = 0;
            let rc = unsafe { uart_poll_in(d, &mut c as *mut u8) };
            if rc < 0 {
                //debug!("uart_poll_in rc={}", rc);
                break; // no more data
            }

            // unsafe {
            //     uart_poll_out(d, c); // echo back for testing
            // }
            
            debug!("Received byte: {}", c);

            if len >= payload.len() {
                break;
            }

            payload[len] = c;
            len += 1;
        }

        len
    }


    fn refresh_connected_flag(&self) {
        let d = self.get_dev();

        if d.is_null() {
            self.connected.store(false, Ordering::Release);
            return;
        }

        let mut dtr: u32 = 0;
        let rc = unsafe { uart_line_ctrl_get(d, UART_LINE_CTRL_DTR, &mut dtr as *mut u32) };

        //debug!("DTR get rc={}, dtr={}", rc, dtr);

        if rc == 0 && dtr != 0 {
            self.connected.store(true, Ordering::Release);
        } else {
            self.connected.store(false, Ordering::Release);
        }
    }


    fn get_dev(&self) -> *const device {
        self.dev_ptr.load(Ordering::Acquire) as *const device
    }
}

impl TransportDriver for UsbSerialDriverZephyr {

    fn init<B>(&mut self, payload: &mut PayloadBuffer<B>) -> bool
    where 
        B: AsRef<[u8]> + AsMut<[u8]>,
    {

        // Enable USB stack
        unsafe {
            trace!("Enabling USB...");
            let ret = usb_enable();
            if ret == 0 {
                ok!("USB enabled");
            } else {
                error!("usb_enable() returned {}", ret);
                return false;
            }
        }

        // Bind CDC ACM UART device
        // Common name: "CDC_ACM_0"
        let name = b"CDC_ACM_0\0";
        trace!("Binding to device \"CDC_ACM_0\"...");
        let d = unsafe { device_get_binding(name.as_ptr() as *const core::ffi::c_char) };
        if d.is_null() {
            // Not ready / misconfigured devicetree/Kconfig
            error!("device_get_binding(\"CDC_ACM_0\") returned NULL");
            self.ready.store(false, Ordering::Release);
            self.dev_ptr.store(0, Ordering::Release);
            // set_dev(ptr::null());
            return false;
        }

        self.dev_ptr.store(d as usize, Ordering::Release);
        self.ready.store(true, Ordering::Release);

        // Wait for a terminal to open the port (DTR asserted).
        // Also assert DCD/DSR for nicer host behavior.
        loop {
            self.refresh_connected_flag();
            if self.connected.load(Ordering::Acquire) {
                unsafe {
                    let rc = uart_line_ctrl_set(d, UART_LINE_CTRL_DCD, 1);
                    // if rc != 0 {
                    //     error!("Failed to set DCD, rc={}", rc);
                    // }
                    let rc = uart_line_ctrl_set(d, UART_LINE_CTRL_DSR, 1);
                    // if rc == 0 {
                    //     error!("Failed to set DSR, rc={}", rc);
                    // }
                }
                break;
            }
            Time::sleep_ms(10);
        }

        payload.clear(); // clear 
        self.poll_rx_once(&mut []); // prime

        ok!("serial initialized");
        true
    }

    fn poll_rx<B>(&mut self, payload: &mut PayloadBuffer<B>) -> Result<(), Error>
    where
        B: AsRef<[u8]> + AsMut<[u8]>,
    {
        if !self.ready.load(Ordering::Acquire) {
            return Ok(());
        }

        self.refresh_connected_flag();

        let mut tmp = [0u8; 256];
        let n = self.poll_rx_once(&mut tmp);
        // trace!("Polled UART, got {} bytes", n);

        if n > 0 {
            debug!("USB RX: {}", hexdump!(&tmp[..n]));
            payload.push(&tmp[..n])?;
        } 

        Ok(())
    }

    // fn has_data(&self) -> bool {
    //     self.poll_if_ready() && self.rx.len() > 0
    // }

    // fn get_data_len() -> usize {
    //     if !poll_if_ready() {
    //         return 0;
    //     }
    //     RX.len()
    // }

    // fn get_data(out: &mut [u8]) -> usize {
    //     if !poll_if_ready() || out.is_empty() {
    //         return 0;
    //     }
    //     RX.read_all(out)
    // }

    // fn has_line() -> bool {
    //     poll_if_ready() && RX.has_line()
    // }

    // fn get_line() -> Option<String> {
    //     if !poll_if_ready() || !RX.has_line() {
    //         return None;
    //     }
    //     RX.get_line()
    // }

    // fn clear_data() {
    //     RX.clear();
    // }

    /// Send reply to the current connected client.
    /// Appends '\n'.
    fn send(&mut self, data: &[u8]) -> bool {
        if !self.ready.load(Ordering::Acquire) {
            error!("Serial not ready, cannot send");
            return false;
        }
        self.refresh_connected_flag();
        if !self.connected.load(Ordering::Acquire) {
            error!("No client connected, cannot send");
            return false;
        }

        let d = self.get_dev();
        if d.is_null() {
            error!("Device disappeared, cannot send");
            return false;
        }

        for &b in data {
            trace!("Sending byte: {}", b);
            unsafe { uart_poll_out(d, b) };
        }
        trace!("Sending byte: {}", b'\n');
        unsafe { uart_poll_out(d, b'\n') };
        true
    }
}

impl UsbSerialTransport<&'static mut [u8]> {
    pub fn take() -> Option<Self> {
        Some(
            Self::new(
                UsbSerialDriverZephyr::new(),
                crate::drivers::payload_storage::take()?, // default payload buffer
            )
        )
    }
}


