use qrcodegen_no_heap::QrCode;
use qrcodegen_no_heap::QrCodeEcc;
use qrcodegen_no_heap::Version;
extern crate alloc;
use alloc::vec;

use core::cell::UnsafeCell;

pub const QR_VERSION_NUM: u8 = 13;
const QR_VERSION: Version = Version::new(QR_VERSION_NUM);
const QR_MAX_WIDTH: u8 = 170; // Maximum width in pixels for the QR code on the display
const QR_MODULES_MAX: usize = QR_WIDTH * QR_WIDTH;
const QR_BITMAP_LEN: usize = QR_MODULES_MAX.div_ceil(8);

use crate::drivers::Display;

const QR_BUF_LEN: usize = QR_VERSION.buffer_len();
const QR_WIDTH: usize = QR_VERSION_NUM as usize * 4 + 17; // Size of the QR code in modules (e.g., version 1 is 21x21, version 25 is 117x117)

struct SingleThreaded<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SingleThreaded<T> {}

static QR_COORDS: SingleThreaded<Option<(u16, u16)>> = SingleThreaded(UnsafeCell::new(None));

static QR_BITMAP: SingleThreaded<[u8; QR_BITMAP_LEN]> =
    SingleThreaded(UnsafeCell::new([0u8; QR_BITMAP_LEN]));
static QR_HAS_DATA: SingleThreaded<bool> =
    SingleThreaded(UnsafeCell::new(false));

fn set_bit(bitmap: &mut [u8], index: usize, value: bool) {
    let byte = index / 8;
    let bit = index % 8;
    if value {
        bitmap[byte] |= 1 << bit;
    } else {
        bitmap[byte] &= !(1 << bit);
    }
}

fn get_bit(bitmap: &[u8], index: usize) -> bool {
    let byte = index / 8;
    let bit = index % 8;
    (bitmap[byte] & (1 << bit)) != 0
}

pub struct QR;

impl QR {
    const QR_ECC_LEVEL: QrCodeEcc = QrCodeEcc::Low; // Adjust error correction level as needed
    pub fn new() -> Self {
        Self {}
    }

    pub fn image_width() -> u16 {
        if Self::has_data() {
            let qr_density = Self::density() as usize;
            (QR::size() as usize * qr_density) as u16
        } else {
            0
        }
    }

    pub fn density() -> u8 {
        if Self::has_data() {
            QR_MAX_WIDTH / QR::size() as u8
        } else {
            0
        }
    }

    pub fn clear() {
        Display::clear_qr();
        unsafe {
            *QR_HAS_DATA.0.get() = false;
            *QR_COORDS.0.get() = None;
        }
    }

    pub fn draw_if_needed() {
        if Self::has_data() {
            Display::draw_qr();
        }
    }

    pub fn get_module(x: u8, y: u8) -> bool {
        unsafe {
            if !*QR_HAS_DATA.0.get() {
                return false;
            }
            let bitmap = &*QR_BITMAP.0.get();
            let index = y as usize * QR_WIDTH + x as usize;
            get_bit(bitmap, index)
        }
    }

    pub fn size() -> u8 {
        if Self::has_data() {
            QR_WIDTH as u8
        } else {
            0
        }
    }

    fn calculate_centered_coords() -> (u16, u16) {
        let y_offset  = 40;
        let x = (Display::WIDTH - Self::image_width()) / 2;
        let y = (Display::HEIGHT - y_offset / 4 - Self::image_width()) / 2;
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
      unsafe { *QR_HAS_DATA.0.get() }
    }

    pub fn create_from_str(data: &str, coords: Option<(u16, u16)>) {
        let version = QR_VERSION;
        let mut dataandtemp = vec![0u8; version.buffer_len()];
        let mut out_buffer = vec![0u8; version.buffer_len()];

        let qr = QrCode::encode_text(
            data,
            &mut dataandtemp,
            out_buffer.as_mut_slice(),
            Self::QR_ECC_LEVEL,
            version,
            version,
            None,
            true,
        );

        let qr = match qr {
            Ok(qr) => qr,
            Err(e) => {
                error!("Failed to create QR code from {}: {}", data, e);
                unsafe {
                    *QR_HAS_DATA.0.get() = false;
                    *QR_COORDS.0.get() = None;
                }
                return;
            }
        };

        unsafe {
            let bitmap = &mut *QR_BITMAP.0.get();
            bitmap.fill(0);

            let size = qr.size() as usize;
            for y in 0..size {
                for x in 0..size {
                    let index = y * QR_WIDTH + x;
                    set_bit(bitmap, index, qr.get_module(x as i32, y as i32));
                }
            }

            *QR_HAS_DATA.0.get() = true;
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
        assert_eq!(QR::size(), QR_WIDTH as u8);
        for y in 0..QR::size() {
            for x in 0..QR::size() {
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
        let long_data = "A".repeat(2000); // 2000 characters, which exceeds the capacity of version 13 with low ECC
        QR::create_from_str(&long_data, None);
        assert!(!QR::has_data(), "QR code should not be created for data that exceeds capacity");
    }
}