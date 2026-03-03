// bluetooth.rs
#![allow(dead_code)]
use crate::String;

use crate::drivers::bluetooth::BluetoothDriver;
use crate::drivers::UsbSerial;

pub struct BluetoothSimulator;

static mut CURRENT_LINE: Option<String> = None;

impl BluetoothDriver for BluetoothSimulator {

    fn init() -> bool  { UsbSerial::init() }

    fn start() -> bool { true }
    fn stop() -> bool  { true }

    #[allow(static_mut_refs)]
    fn get_line() -> Option<&'static str> { unsafe {
        CURRENT_LINE = UsbSerial::get_line();
        Some(CURRENT_LINE.as_ref()?.as_str())
    }}

    fn clear_data() { UsbSerial::clear_data() }

    fn send(data: &[u8]) -> bool { UsbSerial::send(data) }
}


