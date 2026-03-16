#![no_std]
#![allow(dead_code)]

extern crate alloc;

#[macro_use] 
pub mod logging;
pub mod drivers;
//mod protocols;

#[cfg(feature = "simulator")]
mod simulator_window;

#[cfg(feature = "simulator")]
extern crate std;

pub mod allocator;

pub use alloc::string::String;
pub use alloc::vec::Vec;

// macro to simplify main function definition in hito applications
#[macro_export]
macro_rules! hito_main {
    ($app_fn:expr) => {


        extern crate alloc;

        #[cfg(feature = "zephyr")]
        extern "C" {
            fn hito_pin_config();
            fn hito_button_init();
        }

        #[inline (always)]
        pub fn hito_main() {

            #[cfg(feature = "zephyr")]
            unsafe {
                hito_pin_config();
                hito_button_init();
            }

            //#[cfg(feature = "zephyr")]

            $crate::drivers::log_backend::init();
            log_boot_banner!();
            $app_fn();
        }

        #[cfg(feature = "zephyr")]
        #[no_mangle]
        pub extern "C" fn rust_main() -> ! {
            hito_main();
            loop {}
        }

        use $crate::allocator::Allocator;

        #[global_allocator]
        static ALLOCATOR: Allocator = Allocator;

        #[cfg(feature = "zephyr")]
        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            $crate::logging::write($crate::logging::Level::Error, format_args!("{}", info));
            loop {}
        }

        #[cfg(feature = "zephyr")]
        #[no_mangle]
        pub extern "C" fn __aeabi_unwind_cpp_pr0() {
            // Stub for C++ exception unwinding (unused in no_std)
        }

        // TODO add out of memory handler somewhere
        //#[alloc_error_handler]
        //fn oom(_: core::alloc::Layout) -> ! {
            //error!("Out of memory!");
            //loop {}
        //}

    };
}


