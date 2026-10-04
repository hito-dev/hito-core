use crate::driver;

driver! {
    pub trait TouchDriver => Touch {
        fn init() -> bool;   
        //fn has_touch() -> bool;   
        fn xy() -> Option<(u16, u16)>;
    }
}

#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::touch::TouchMinifb as Touch;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::touch::TouchZephyr as Touch;

