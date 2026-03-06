use qrcodegen_no_heap::QrCode;
use qrcodegen_no_heap::QrCodeEcc;
use qrcodegen_no_heap::Version;
extern crate alloc;
use alloc::vec;

use core::cell::UnsafeCell;

pub const QR_VERSION_NUM: u8 = 25; // We're using version 25 which can hold up to 1064 bytes of data with low error correction
const QR_VERSION: Version = Version::new(QR_VERSION_NUM);

use crate::drivers::Display;

const QR_BUF_LEN: usize = QR_VERSION.buffer_len();
//const QR_BUF_LEN: usize = Version::MAX.buffer_len(); // 3706 bytes, enough for version 40 with low ECC, which is the largest possible QR code. We can use a smaller buffer if we want to limit the max version.
const QR_WIDTH: usize = QR_VERSION_NUM as usize * 4 + 17; // Size of the QR code in modules (e.g., version 1 is 21x21, version 25 is 117x117)

struct SingleThreaded<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SingleThreaded<T> {}

static QR_CODE: SingleThreaded<Option<QrCode>> = SingleThreaded(UnsafeCell::new(None));
static QR_COORDS: SingleThreaded<Option<(u16, u16)>> = SingleThreaded(UnsafeCell::new(None));
static OUT_BUFFER: SingleThreaded<[u8; QR_BUF_LEN]> = SingleThreaded(UnsafeCell::new([0u8; QR_BUF_LEN]));

pub struct QR;

impl QR {
    const QR_ECC_LEVEL: QrCodeEcc = QrCodeEcc::Low; // Adjust error correction level as needed
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_module(x: i32, y: i32) -> bool {
        let qr = unsafe { &*QR_CODE.0.get() };
        if qr.is_none() {
            return false;
        }
        let qr = &*qr.as_ref().unwrap();
        qr.get_module(x, y)
    }

    pub fn width() -> u8 {
        if Self::has_data() {
            QR_WIDTH as u8
        } else {
            0
        }
    }


    fn calculate_centered_coords() -> (u16, u16) {
        let y_offset  = 40;
        let x = (Display::WIDTH - QR::width() as u16) / 2;
        let y = (Display::HEIGHT - y_offset / 4 - QR::width() as u16) / 2;
        (x, y)
    }

    pub fn get_coords() -> (u16, u16) {
        if Self::has_data() {
            unsafe { (*QR_COORDS.0.get()).clone().unwrap_or_else(Self::calculate_centered_coords) }
        } else {
            Self::calculate_centered_coords()
        }
    }

    pub fn is_centered() -> bool { true }

    pub fn has_data() -> bool {
      unsafe { (*QR_CODE.0.get()).is_some() }
    }

    pub fn create_from_str(data: &str, coords: Option<(u16, u16)>) {
        let version = QR_VERSION;
        let mut dataandtemp = vec![0u8; version.buffer_len()];

        unsafe {
            let outbuffer_ref = &mut *OUT_BUFFER.0.get();
            let qr = QrCode::encode_text(data,
            &mut dataandtemp, outbuffer_ref, Self::QR_ECC_LEVEL,
            version, version, None, true);
            if qr.is_err() {
                error!("Failed to create QR code from {}: {}",data, qr.err().unwrap());
                return;
            }
            *QR_CODE.0.get() = Some(qr.unwrap());
            *QR_COORDS.0.get() = coords;
        }
    }
}


#[cfg(test)]
mod qr_tests {
    use super::*;
    #[test]
    fn test_qr_creation() {
        use std::{println, print};
        QR::create_from_str("Hello, world!", None);
        assert!(QR::has_data());
        assert_eq!(QR::width(), QR_WIDTH as u8);
        for y in 0..QR_WIDTH as i32 {
            for x in 0..QR_WIDTH as i32 {
                let module = QR::get_module(x, y);

                if module {
                    print!("█");
                } else {
                    print!("  ");
                }
            }
            println!();
        }
    }

    #[test]
    fn test_long_qr_should_not_be_created() {
        let long_data = "A".repeat(2000); // 2000 characters, which exceeds the capacity of version 25 with low ECC
        QR::create_from_str(&long_data, None);
        assert!(!QR::has_data(), "QR code should not be created for data that exceeds capacity");
    }
}