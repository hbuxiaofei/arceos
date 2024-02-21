use core::alloc::Layout;
use core::ptr::NonNull;
use core::cmp::max;
use core::mem::size_of;

use crate::{AllocError, AllocResult};

pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    base: usize,
    cap: usize,
    forward: usize,
    backward: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            base: 0,
            cap: 0,
            forward: 0,
            backward: 0,
        }
    }

    pub fn init(&mut self, start: usize, size: usize) {
        self.base = start;
        self.cap = size;

        self.forward = self.base;
        self.backward = self.base + self.cap;
    }

    pub fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let addr = self.forward;
        self.forward += size;

        if self.forward > self.backward {
            return Err(AllocError::NoMemory);
        }

        unsafe {
            Ok(NonNull::new_unchecked(addr as *mut u8))
        }
    }

    pub fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
    }

    pub fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }

        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        let align_log2 = align_pow2.trailing_zeros() as usize;

        self.backward -= num_pages * PAGE_SIZE;
        self.backward = (((self.backward + 1) >> align_log2) - 1) << align_log2;

        if self.forward > self.backward {
            return Err(AllocError::NoMemory);
        }

        Ok(self.backward)
    }

    pub fn dealloc_pages(&self, _pos: usize, _num_pages: usize) {
    }

    pub fn used_bytes(&self) -> usize {
        self.cap - self.available_bytes()
    }

    pub fn available_bytes(&self) -> usize {
        self.backward - self.forward
    }

    pub fn used_pages(&self) -> usize {
        self.used_bytes() / PAGE_SIZE
    }

    pub fn available_pages(&self) -> usize {
        self.available_bytes() / PAGE_SIZE
    }
}
