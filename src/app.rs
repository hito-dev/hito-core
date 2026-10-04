use crate::drivers;

pub fn run() {

    drivers::log_backend::init();

    ok!("Rust main started {}", "with args");
    error!("Some error occured");
    warn!("warn test");
    info!("info test");
    d!("d/debug test");
    debug!("debug test");
    t!("t/trace test");
    trace!("trace test");

}

//#[cfg(feature = "zephyr")]
//extern crate panic_halt;

// ARM EABI unwinding stub for embedded targets only
//#[cfg(feature = "zephyr")]
//#[no_mangle]
//pub extern "C" fn __aeabi_unwind_cpp_pr0() {
//    // Stub for C++ exception unwinding (unused in no_std)
//}
