use bump::BumpAllocator;
// use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub mod bump;

// The responsibility of an allocator is to manage the available heap memory.
// It needs to return unused memory on alloc calls and keep track of memory
// freed by dealloc so that it can be reused again. Most importantly, it
// must never hand out memory that is already in use somewhere else because
// this would cause undefined behavior.

// Apart from correctness, there are many secondary design goals. For example,
// the allocator should effectively utilize the available memory and keep
// fragmentation low. Furthermore, it should work well for concurrent
// applications and scale to any number of processors. For maximal performance,
// it could even optimize the memory layout with respect to the CPU caches to
// improve cache locality and avoid false sharing.

/// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize {
    // Since align is a power of two, its binary representation has only a single bit set (e.g. 0b000100000).
    // This means that align - 1 has all the lower bits set (e.g. 0b00011111). By creating the bitwise NOT
    // through the ! operator, we get a number that has all the bits set except for the bits lower than align
    // (e.g. 0b…111111111100000). By performing a bitwise AND on an address and !(align - 1), we align the
    // address downwards. This works by clearing all the bits that are lower than align. Since we want to
    // align upwards instead of downwards, we increase the addr by align - 1 before performing the bitwise
    // AND. This way, already aligned addresses remain the same while non-aligned addresses are rounded to
    // the next alignment boundary.
    (addr + align - 1) & !(align - 1)
}

// // Alternative less efficient, but a bit easier to understand align_up fn:
// fn align_up(addr: usize, align: usize) -> usize {
//     let remainder = addr % align;
//     if remainder == 0 {
//         addr
//     } else {
//         addr - remainder + align
//     }
// }

#[global_allocator]
// static ALLOCATOR: LockedHeap = LockedHeap::empty();
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe { ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE) };

    Ok(())
}
