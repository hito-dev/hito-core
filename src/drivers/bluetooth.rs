use crate::driver;

/*
pub trait Transport {

    fn init() -> bool;   
    fn start() -> bool;  
    fn stop() -> bool;   

    fn has_line() -> bool;           
    fn get_line() -> Option<String>; 
}
*/

driver! {
    pub trait BluetoothDriver => Bluetooth {
        fn init() -> bool;   
        fn start() -> bool;  
        fn stop() -> bool;   

        //fn has_data() -> bool;
        //fn get_data_len() -> usize;
        //fn get_data(out: &mut [u8]) -> usize;
        
        //fn has_line() -> bool;           
        // modity to str
        fn get_line() -> Option<&'static str>; 

        fn clear_data();                 

        fn send(data: &[u8]) -> bool;   
        //fn send(str: &str) -> bool;      
    }
}

#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::bluetooth::BluetoothSimulator as Bluetooth;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::bluetooth::BluetoothZephyr as Bluetooth;

