// // bluetooth.rs
// #![allow(dead_code)]
// use core::str;

// use crate::drivers::bluetooth::BluetoothDriver;
// use crate::drivers::time::Time;

use crate::drivers::{
    payload::{PayloadBuffer, Error},
    transport::{TransportDevice, TransportDriver}
};

pub type BluetoothZephyrTransport<B> = TransportDevice<BluetoothZephyrDriver, B>;

pub struct BluetoothZephyrDriver {
}

impl BluetoothZephyrDriver {
    pub fn new() -> Self {
        Self {
        }
    }
}

// pub struct BluetoothZephyr;

// extern "C" {
//     fn hito_ble_init(buf: *mut u8, size: usize) -> bool;
//     fn hito_ble_start();
//     fn hito_ble_stop();

//     fn hito_ble_has_packet() -> bool;
//     fn hito_ble_packet() -> *mut core::ffi::c_void;
//     fn hito_ble_packet_len() -> u16;
//     fn hito_ble_packet_clear();

//     fn hito_ble_has_payload() -> bool;
//     fn hito_ble_payload() -> *const u8;
//     fn hito_ble_payload_len() -> u16;
//     fn hito_ble_payload_clear();

//     fn hito_ble_send(data: *const u8, len: u32) -> bool;
//     fn hito_ble_is_active() -> bool;
// }

// impl BluetoothDriver for BluetoothZephyr {

//     fn init() -> bool  { 
//         use crate::drivers::payload_buffer;
//         unsafe { hito_ble_init(payload_buffer::as_mut_ptr(), payload_buffer::capacity()) }
//     }

//     fn start() -> bool { 
//         unsafe { hito_ble_start(); }
//         true
//     }
//     fn stop() -> bool  { 
//         unsafe { hito_ble_stop(); }
//         true
//     }
//     fn is_active() -> bool { 
//         unsafe { hito_ble_is_active() }
//     }

//     fn get_line() -> Option<&'static str> { 
//         unsafe {
//             if !hito_ble_has_payload() {
//                 return None;
//             }
//             let len = hito_ble_payload_len() as usize;
//             if len == 0 {
//                 return None;
//             }
//             trace!("BluetoothZephyr::get_line len={}", len);
//             let ptr = hito_ble_payload();
//             if ptr.is_null() {
//                 return None;
//             }
//             let bytes = core::slice::from_raw_parts(ptr, len);
//             //if bytes[len - 1] != b'\n' && bytes[len - 1] != b'\0' {
//             //   return None;
//             //}
//             trace!("Raw data: \n{}", hexdump!(bytes));
//             str::from_utf8(bytes).ok()
//         }
//     }

//     fn clear_data() {
//         unsafe { 
//             hito_ble_packet_clear(); 
//             hito_ble_payload_clear(); 
//         }
//     }

//     fn send(data: &[u8]) -> bool {
//         //unsafe { hito_ble_send(data.as_ptr(), data.len() as u32) }
//         const CMD_UPLOAD_INFO: u8 = 0x69; // 'i'
//         const CMD_DATA: u8 = 0x64;        // 'd'
//         const CHUNK_MAX: usize = 504;

//         // ---- 1) upload info ----
//         //let payload_size: u32 = payload
//             //.len()
//             //.try_into()
//             //.map_err(|_| UploadError::Link("payload too large"))?;
//         let payload_size = data.len() as u32;

//         let mut upload_info = [0u8; 5];
//         upload_info[0] = CMD_UPLOAD_INFO;
//         upload_info[1] = (payload_size >> 0) as u8;
//         upload_info[2] = (payload_size >> 8) as u8;
//         upload_info[3] = (payload_size >> 16) as u8;
//         upload_info[4] = (payload_size >> 24) as u8;

//         if !BluetoothZephyr::send_and_wait_ok_ack(&upload_info, 1_000) {
//             warn!("Failed to send upload info");
//             return false;
//         }
//         trace!("Upload info sent, payload size: {}", payload_size);

//         // ---- 2) data chunks ----
//         // Buffer for "d" + chunk
//         let mut pkt = [0u8; 1 + CHUNK_MAX];
//         pkt[0] = CMD_DATA;

//         let mut offset = 0usize;
//         while offset < data.len() {
//             let rest = data.len() - offset;
//             let chunk_len = if rest > CHUNK_MAX { CHUNK_MAX } else { rest };

//             pkt[1..1 + chunk_len].copy_from_slice(&data[offset..offset + chunk_len]);

//             // Only send the used prefix.
//             let timeout = if rest > CHUNK_MAX { 5_000 } else { 1_000 };
//             trace!("Sending data chunk at offset {}, len {}", offset, chunk_len);
//             if !BluetoothZephyr::send_and_wait_ok_ack(&pkt[..1 + chunk_len], timeout) {
//                 warn!("Failed to send data chunk at offset {}", offset);
//                 return false;
//             }

//             offset += chunk_len;
//         }
//         true
//     }

// }

// impl BluetoothZephyr {

//     fn ble_take_buffer() -> Option<&'static [u8]> {
//         unsafe {
//             let ptr = hito_ble_packet();
//             if ptr.is_null() {
//                 return None;
//             }

//             let len = hito_ble_packet_len() as usize;
//             if len == 0 || len > 1024 {
//                 return None;
//             }

//             Some(core::slice::from_raw_parts(ptr as *const u8, len))
//         }
//     }

//     fn is_ok(buf: &[u8]) -> bool {
//         let mut start = 0;
//         let mut end = buf.len();

//         while start < end && matches!(buf[start], b' ' | b'\t' | 0) {
//             start += 1;
//         }
//         while end > start && matches!(buf[end - 1], b'\r' | b'\n' | b' ' | b'\t' | 0) {
//             end -= 1;
//         }

//         &buf[start..end] == b"ok"
//     }

//     fn send_and_wait_ok_ack(data: &[u8], timeout_ms: u32) -> bool {
//         trace!("BluetoothZephyr::send_and_wait_ok_ack");
//         unsafe {
//             trace!("send_and_wait_ok_ack: \n{}", hexdump!(data));
//             hito_ble_packet_clear();
//             hito_ble_send(data.as_ptr(), data.len() as u32);

//             let time = Time::now_ms();
//             while !hito_ble_has_packet() {
//                 if Time::now_ms() - time > timeout_ms as u64 {
//                     trace!("Timeout waiting for ack");
//                     return false;
//                 }
//             }
//             if let Some(buf) = BluetoothZephyr::ble_take_buffer() {
//                 trace!("ack: \n{}", hexdump!(buf));
//                 return BluetoothZephyr::is_ok(buf);
//             } else {
//                 trace!("Failed to take BLE buffer for ack");
//                 return false;
//             }
//         }
//     }
// }
