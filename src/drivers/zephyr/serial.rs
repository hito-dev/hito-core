// serial.rs
#![allow(dead_code)]
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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

// ----- SerialZephyr implementation -----

pub struct SerialZephyr;

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
        let current_len = self.current_len.load(Ordering::Relaxed);
        if current_len >= payload.len() {
            debug!("Payload buffer full, dropping byte");
            return false;
        }

        payload[current_len] = b;
        self.current_len.store(current_len + 1, Ordering::Release);
        true
    }


    fn refresh_connected_flag(&self) {
        let d = self.get_dev();

        if d.is_null() {
            self.connected.store(false, Ordering::Release);
            return;
        }

        let mut dtr: u32 = 0;
        let rc = unsafe { uart_line_ctrl_get(d, UART_LINE_CTRL_DTR, &mut dtr as *mut u32) };

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
                    let _rc = uart_line_ctrl_set(d, UART_LINE_CTRL_DCD, 1);
                    // if rc != 0 {
                    //     error!("Failed to set DCD, rc={}", rc);
                    // }
                    let _rc = uart_line_ctrl_set(d, UART_LINE_CTRL_DSR, 1);
                    // if rc == 0 {
                    //     error!("Failed to set DSR, rc={}", rc);
                    // }
                }
                break;
            }
            Time::sleep_ms(10);
        }

        payload.clear(); // clear 
        //self.poll_rx_once(&mut []); // prime

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
            //trace!("Sending byte: {}", b);
            unsafe { uart_poll_out(d, b) };
        }
        //trace!("Sending byte: {}", b'\n');
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


