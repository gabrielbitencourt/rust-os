use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

#[global_allocator]
static ALLOCATOR: Dummy = Dummy;

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        return null_mut();
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("no dealloc bitch");
    }
}