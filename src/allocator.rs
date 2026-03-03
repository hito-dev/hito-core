use core::alloc::{GlobalAlloc, Layout};

#[cfg(feature = "zephyr")]
extern "C" {
    fn k_malloc(size: usize) -> *mut u8;
    fn k_free(ptr: *mut u8);
}

#[cfg(feature = "simulator")]
extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        #[cfg(feature = "zephyr")]
        return unsafe { k_malloc(layout.size()) };

        #[cfg(feature = "simulator")]
        return unsafe { malloc(layout.size()) };
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        #[cfg(feature = "zephyr")]
        unsafe { k_free(ptr) };

        #[cfg(feature = "simulator")]
        unsafe { free(ptr) };
    }
}

