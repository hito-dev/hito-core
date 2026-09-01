// nfc.rs
#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;
use core::ffi::c_int;

use crate::drivers::{
    payload::{Error, PayloadBuffer},
    transport::{TransportDevice, TransportDriver},
};

pub type NfcTransport<B> = TransportDevice<NfcZephyrDriver, B>;

pub struct NfcZephyrDriver;

impl NfcZephyrDriver {
    pub const fn new() -> Self {
        Self
    }
}

impl TransportDriver for NfcZephyrDriver {
    fn stop(&mut self) -> bool {
        unsafe {
            hito_nfc_stop();
        }

        true
    }

    fn init<B>(&mut self, _payload: &mut PayloadBuffer<B>) -> bool
    where
        B: AsRef<[u8]> + AsMut<[u8]>,
    {
        // The C implementation calls strlen(), so the message must
        // always be null-terminated.
        static EMPTY_MESSAGE: &[u8] = b"\0";

        unsafe { hito_nfc_start(EMPTY_MESSAGE.as_ptr()) }
    }

    fn poll_rx<B>(&mut self, payload: &mut PayloadBuffer<B>) -> Result<(), Error>
    where
        B: AsRef<[u8]> + AsMut<[u8]>,
    {
        unsafe {
            if !hito_nfc_has_data() {
                return Ok(());
            }

            let len = hito_nfc_data_len() as usize;
            let data_ptr = hito_nfc_data();

            if data_ptr.is_null() || len == 0 {
                hito_nfc_data_clear();
                return Ok(());
            }

            let data = core::slice::from_raw_parts(data_ptr, len);
            let start_from = if data.len() >= 3
                && (data[0] == 0x02 || (data[0] == b'e' && data[1] == b'n'))
            {
                3
            } else {
                0
            };
            let data = &data[start_from..];

            if data.len() > payload.capacity() {
                warn!(
                    "NFC payload does not fit into the receive buffer: {} > {}",
                    data.len(),
                    payload.capacity()
                );

                // Drop the packet because it cannot be represented by
                // the provided PayloadBuffer.
                hito_nfc_data_clear();
                return Ok(());
            }

            let storage = payload.storage_mut().as_mut();

            storage[..data.len()].copy_from_slice(data);
            payload.set_len(data.len());

            // Clear the C-side state only after copying the data.
            hito_nfc_data_clear();
        }

        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> bool {
        // hito_nfc_set_message() uses strlen(), so embedded null bytes
        // cannot be represented.
        if data.contains(&0) {
            warn!("NFC message contains an embedded null byte");
            return false;
        }

        // Add the null terminator required by the unchanged C API.
        let mut message = Vec::with_capacity(data.len() + 1);
        message.extend_from_slice(data);
        message.push(0);

        // The C function synchronously copies and encodes the message,
        // so the temporary Vec can be dropped after this call.
        unsafe { hito_nfc_set_message(message.as_ptr()) == 0 }
    }
}

impl NfcTransport<&'static mut [u8]> {
    pub fn take() -> Option<Self> {
        Some(Self::new(
            NfcZephyrDriver::new(),
            crate::drivers::payload_storage::take()?,
        ))
    }
}

extern "C" {
    fn hito_nfc_start(message: *const u8) -> bool;
    fn hito_nfc_stop();

    // The C implementation returns int.
    fn hito_nfc_set_message(message: *const u8) -> c_int;

    fn hito_nfc_has_data() -> bool;
    fn hito_nfc_data_len() -> u32;
    fn hito_nfc_data() -> *const u8;
    fn hito_nfc_data_clear();
}
