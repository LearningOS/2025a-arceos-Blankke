#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator, AllocResult, AllocError};
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,      // bytes allocation position (forward)
    p_pos: usize,      // pages allocation position (backward)
    count: usize,      // number of byte allocations
    bytes_start: usize, // start of bytes area for tracking
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
            bytes_start: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.count = 0;
        self.bytes_start = start;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        // Bump allocator doesn't support adding memory after initialization
        Err(AllocError::NoMemory)
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();

        // Align the current position
        let aligned_pos = (self.b_pos + align - 1) & !(align - 1);
        let new_pos = aligned_pos + size;

        // Check if we have enough space (ensure we don't collide with pages area)
        if new_pos > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        self.b_pos = new_pos;
        self.count += 1;

        NonNull::new(aligned_pos as *mut u8).ok_or(AllocError::InvalidParam)
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: core::alloc::Layout) {
        // Decrease count and reset bytes area when count reaches 0
        if self.count > 0 {
            self.count -= 1;
            if self.count == 0 {
                // Reset bytes area
                self.b_pos = self.bytes_start;
            }
        }
    }

    fn total_bytes(&self) -> usize {
        if self.end > self.start {
            self.end - self.start
        } else {
            0
        }
    }

    fn used_bytes(&self) -> usize {
        if self.b_pos > self.bytes_start {
            self.b_pos - self.bytes_start + (self.end - self.p_pos)
        } else {
            self.end - self.p_pos
        }
    }

    fn available_bytes(&self) -> usize {
        if self.p_pos > self.b_pos {
            self.p_pos - self.b_pos
        } else {
            0
        }
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> AllocResult<usize> {
        let total_size = num_pages * SIZE;
        
        // Align backward from p_pos
        let aligned_pos = (self.p_pos - total_size) & !(align_pow2 - 1);
        
        // Check if we have enough space (ensure we don't collide with bytes area)
        if aligned_pos < self.b_pos {
            return Err(AllocError::NoMemory);
        }

        self.p_pos = aligned_pos;
        Ok(aligned_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Pages are never freed according to the comment
        // "For pages area, it will never be freed!"
    }

    fn total_pages(&self) -> usize {
        if self.end > self.start {
            (self.end - self.start) / SIZE
        } else {
            0
        }
    }

    fn used_pages(&self) -> usize {
        if self.end > self.p_pos {
            (self.end - self.p_pos) / SIZE
        } else {
            0
        }
    }

    fn available_pages(&self) -> usize {
        if self.p_pos > self.b_pos {
            (self.p_pos - self.b_pos) / SIZE
        } else {
            0
        }
    }
}