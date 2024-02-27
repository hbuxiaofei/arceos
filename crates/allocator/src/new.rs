use linked_list_allocator::Heap;

use core::alloc::Layout;
use core::ptr::NonNull;
use crate::{AllocError, AllocResult, BaseAllocator, ByteAllocator};

pub struct YourNewByteAllocator {
    inner: Option<Heap>,
}

impl YourNewByteAllocator {
    pub const fn new() -> Self {
        Self { inner: None }
    }

    fn inner_mut(&mut self) -> &mut Heap {
        self.inner.as_mut().unwrap()
    }

    fn inner(&self) -> &Heap {
        self.inner.as_ref().unwrap()
    }
}

impl BaseAllocator for YourNewByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.inner = unsafe { Some(Heap::new(start as *mut u8, size)) };
    }

    fn add_memory(&mut self, _start: usize, size: usize) -> AllocResult {
        unsafe { self.inner_mut().extend(size) };
        Ok(())
    }
}

impl ByteAllocator for YourNewByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        self.inner_mut().allocate_first_fit(layout).map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        unsafe { self.inner_mut().deallocate(pos, layout) }
    }

    fn total_bytes(&self) -> usize {
        self.inner().size()
    }

    fn used_bytes(&self) -> usize {
        self.inner().used()
    }

    fn available_bytes(&self) -> usize {
        self.inner().free()
    }
}
