// bluetooth.rs
#![allow(dead_code)]
use crate::String;

use crate::drivers::bluetooth::BluetoothDriver;
use crate::drivers::UsbSerial;

pub struct BluetoothSimulator;

//use crate::drivers::time::Time;

static mut CURRENT_LINE: Option<String> = None;

static mut IS_ACTIVE: bool = false;

impl BluetoothDriver for BluetoothSimulator {

    fn init() -> bool  { UsbSerial::init() }

    fn start() -> bool { 
        unsafe { IS_ACTIVE = true; }
        true 
    }
    fn stop() -> bool  { 
        unsafe { IS_ACTIVE = false; }
        true 
    }

    fn is_active() -> bool { 
        unsafe { IS_ACTIVE }
    }

    #[allow(static_mut_refs)]
    fn get_line() -> Option<&'static str> { unsafe {
        CURRENT_LINE = UsbSerial::get_line();
        Some(CURRENT_LINE.as_ref()?.as_str())
    }}

    fn clear_data() { UsbSerial::clear_data() }

    fn send(data: &[u8]) -> bool { UsbSerial::send(data) }
}


