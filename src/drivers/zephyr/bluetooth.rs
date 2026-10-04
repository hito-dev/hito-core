// bluetooth.rs
#![allow(dead_code)]
use core::str;

// use crate::drivers::bluetooth::BluetoothDriver;
use crate::drivers::time::Time;

use crate::drivers::{
    payload::{PayloadBuffer, Error},
    transport::{TransportDevice, TransportDriver}
};

pub type BluetoothTransport<B> = TransportDevice<BluetoothZephyrDriver, B>;

pub struct BluetoothZephyrDriver {
}

impl BluetoothZephyrDriver {
    pub fn new() -> Self {
        Self {
        }
    }

    fn send_and_wait_ok_ack(&mut self, data: &[u8], timeout_ms: u32) -> bool {
        trace!("BluetoothZephyr::send_and_wait_ok_ack");
        unsafe {
            trace!("send_and_wait_ok_ack: \n{}", hexdump!(data));
            hito_ble_packet_clear();
            hito_ble_send(data.as_ptr(), data.len() as u32);

            let time = Time::now_ms();
            while !hito_ble_has_packet() {
                if Time::now_ms() - time > timeout_ms as u64 {
                    trace!("Timeout waiting for ack");
                    return false;
                }
            }
            let len = unsafe { hito_ble_packet_len() } as usize;
            if len > 2 {
                let ptr = unsafe { hito_ble_packet() } as *mut u8;
                let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
                if slice == b"ok\n" || slice == b"ok\r\n" || slice == b"ok\0" {
                    trace!("Received ack: \n{}", hexdump!(slice));
                    return true;
                } else {
                    trace!("Received ack is not 'ok': \n{}", hexdump!(slice));
                    return false;
                }
            }
        }
        false
    }

}

impl TransportDriver for BluetoothZephyrDriver {
    fn stop(&mut self) -> bool {
        unsafe {
            hito_ble_stop();
            hito_ble_packet_clear();
            !hito_ble_is_active()
        }
    }

    fn init<B>(&mut self, payload: &mut PayloadBuffer<B>) -> bool
    where 
        B: AsRef<[u8]> + AsMut<[u8]>,
    {
        unsafe { hito_ble_init(payload.storage_mut().as_mut_ptr(), payload.capacity()) }
    }

    /// Poll for incoming data
    fn poll_rx<B>(&mut self, payload: &mut PayloadBuffer<B>) -> Result<(), Error>
    where
        B: AsRef<[u8]> + AsMut<[u8]>,
    {
        // bluetooth driver is polled in the background by C driver, so no polling needed
        if unsafe { hito_ble_has_payload() } {
            payload.set_len(unsafe { hito_ble_payload_len() as usize });
            unsafe {
                hito_ble_payload_clear();
            }
            return Ok(());
        }
        Ok(())
    }

    /// Send reply to the current connected client.
    fn send(&mut self, data: &[u8]) -> bool {
         const CMD_UPLOAD_INFO: u8 = 0x69; // 'i'
         const CMD_DATA: u8 = 0x64;        // 'd'
         const CHUNK_MAX: usize = 504;

         // ---- 1) upload info ----
         //let payload_size: u32 = payload
             //.len()
             //.try_into()
             //.map_err(|_| UploadError::Link("payload too large"))?;
         let payload_size = data.len() as u32;

         let mut upload_info = [0u8; 5];
         upload_info[0] = CMD_UPLOAD_INFO;
         upload_info[1] = (payload_size >> 0) as u8;
         upload_info[2] = (payload_size >> 8) as u8;
         upload_info[3] = (payload_size >> 16) as u8;
         upload_info[4] = (payload_size >> 24) as u8;

         if !self.send_and_wait_ok_ack(&upload_info, 1_000) {
             warn!("Failed to send upload info");
             return false;
         }
         trace!("Upload info sent, payload size: {}", payload_size);

         // ---- 2) data chunks ----
         // Buffer for "d" + chunk
         let mut pkt = [0u8; 1 + CHUNK_MAX];
         pkt[0] = CMD_DATA;

         let mut offset = 0usize;
         while offset < data.len() {
             let rest = data.len() - offset;
             let chunk_len = if rest > CHUNK_MAX { CHUNK_MAX } else { rest };

             pkt[1..1 + chunk_len].copy_from_slice(&data[offset..offset + chunk_len]);

             // Only send the used prefix.
             let timeout = if rest > CHUNK_MAX { 5_000 } else { 1_000 };
             trace!("Sending data chunk at offset {}, len {}", offset, chunk_len);
             if !self.send_and_wait_ok_ack(&pkt[..1 + chunk_len], timeout) {
                 warn!("Failed to send data chunk at offset {}", offset);
                 return false;
             }

             offset += chunk_len;
         }
         true
     }

}

impl BluetoothTransport<&'static mut [u8]> {
    pub fn take() -> Option<Self> {
        Some(
            Self::new(
                BluetoothZephyrDriver::new(),
                crate::drivers::payload_storage::take()?, // default payload buffer
            )
        )
    }
}

extern "C" {
    fn hito_ble_init(buf: *mut u8, size: usize) -> bool;
    fn hito_ble_start();
    fn hito_ble_stop();

    fn hito_ble_has_packet() -> bool;
    fn hito_ble_packet() -> *mut core::ffi::c_void;
    fn hito_ble_packet_len() -> u16;
    fn hito_ble_packet_clear();

    fn hito_ble_has_payload() -> bool;
    fn hito_ble_payload() -> *const u8;
    fn hito_ble_payload_len() -> u16;
    fn hito_ble_payload_clear();

    fn hito_ble_send(data: *const u8, len: u32) -> bool;
    fn hito_ble_is_active() -> bool;
}

