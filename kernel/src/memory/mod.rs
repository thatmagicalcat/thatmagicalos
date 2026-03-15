use core::ops::Range;

use multiboot2::MemoryAreaType;

use crate::kernel_bounds;
pub use bitmapframealloc::BitmapFrameAllocator;
pub use tinyalloc::TinyAllocator;

mod bitmapframealloc;
mod tinyalloc;
pub mod paging;

pub const PAGE_SIZE: usize = 1024 * 4;

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame(usize);

impl Frame {
    pub const fn get_ptr(&self) -> *mut u8 {
        (self.0 * PAGE_SIZE) as *mut u8
    }

    pub const fn start_address(&self) -> usize {
        self.0 * PAGE_SIZE
    }

    pub const fn end_address(&self) -> usize {
        (self.0 + 1) * PAGE_SIZE
    }

    pub const fn from_addr(addr: usize) -> Self {
        Self(addr / PAGE_SIZE)
    }
}
