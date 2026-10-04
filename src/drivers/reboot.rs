use crate::driver;

driver! {
    pub trait RebootDriver => Reboot {
        fn reboot();
    }
}

#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::reboot::RebootSimulator as Reboot;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::reboot::RebootZephyr as Reboot;