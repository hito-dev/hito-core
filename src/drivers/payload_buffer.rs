const MAX_PAYLOAD: usize = 34768;

use core::cell::UnsafeCell;

struct PayloadBuffer(UnsafeCell<[u8; MAX_PAYLOAD]>);
unsafe impl Sync for PayloadBuffer {} // SAFETY: The PayloadBuffer is only accessed through safe
                                      // APIs that ensure proper synchronization.

static PAYLOAD: PayloadBuffer = PayloadBuffer(UnsafeCell::new([0u8; MAX_PAYLOAD]));

//pub fn as_mut() -> &'static mut [u8] {
    //unsafe { &mut PAYLOAD }
//}

pub fn as_mut_ptr() -> *mut u8 {
    PAYLOAD.0.get() as *mut u8
}

pub fn capacity() -> usize {
    MAX_PAYLOAD
}
