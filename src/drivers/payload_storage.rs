use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub const PAYLOAD_CAPACITY: usize = 34_768;

pub struct PayloadStorage {
    taken: AtomicBool,
    buf: UnsafeCell<[u8; PAYLOAD_CAPACITY]>,
}

// empty - we are single threaded embedded environment atm
unsafe impl Sync for PayloadStorage {}

impl PayloadStorage {
    pub const fn new() -> Self {
        Self {
            taken: AtomicBool::new(false),
            buf: UnsafeCell::new([0; PAYLOAD_CAPACITY]),
        }
    }

    pub fn take(&self) -> Option<&mut [u8]> {
        if self.taken.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            Some(unsafe { &mut *self.buf.get() })
        } else {
            None
        }
    }

    pub fn release(&self) {
        self.taken.store(false, Ordering::Release);
    }
}

static PAYLOAD_STORAGE: PayloadStorage = PayloadStorage::new();

pub fn take() -> Option<&'static mut [u8]> {
    PAYLOAD_STORAGE.take()
}
