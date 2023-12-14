use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// Initialize a new OffsetPageTable.
///
/// # Safety
/// This function is unsafe because the caller must guaruntee that the
/// complete physical memory is mapped to the virtual memory at the passed
/// `physical_memory_offset`. Also, this function must only be called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);

    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guaruntee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also this function must only be called
/// once to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (leve_4_table_frame, _) = Cr3::read();
    let phys = leve_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}
// We don’t need to use an unsafe block here because Rust treats the complete body of an unsafe fn
// like a large unsafe block. This makes our code more dangerous since we could accidentally introduce
// an unsafe operation in previous lines without noticing. It also makes it much more difficult to
// spot unsafe operations in between safe operations. There is an RFC to change this behavior.

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // Get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);

        // Map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());

        // Transform to an iterator of frame start addressses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        // Create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    // We first use the usable_frames method to get an iterator of usable frames from the
    // memory map. Then, we use the Iterator::nth function to get the frame with index
    // self.next (thereby skipping (self.next - 1) frames). Before returning that frame,
    // we increase self.next by one so that we return the following frame on the next call.
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;

        frame
    }
}

// /// Creates an example mapping for the given page to frame `0xb8000`.
// pub fn create_example_mapping(
//     page: Page,
//     mapper: &mut OffsetPageTable,
//     frame_allocator: &mut impl FrameAllocator<Size4KiB>,
// ) {
//     use x86_64::structures::paging::PageTableFlags as Flags;

//     let frame = PhysFrame::containing_address(PhysAddr::new(0xB8000));
//     let flags = Flags::PRESENT | Flags::WRITABLE;
//     let map_to_result = unsafe {
//         // FIXME: this is not safe, we do it only for testing.
//         mapper.map_to(page, frame, flags, frame_allocator)
//     };

//     map_to_result.expect("map_to failed").flush()
// }
// // In addition to the page that should be mapped, the function expects a mutable reference to an
// // OffsetPageTable instance and a frame_allocator. The frame_allocator parameter uses the impl Trait
// // syntax to be generic over all types that implement the FrameAllocator trait. The trait is generic
// // over the PageSize trait to work with both standard 4 KiB pages and huge 2 MiB/1 GiB pages. We
// // only want to create a 4 KiB mapping, so we set the generic parameter to Size4KiB.

// /// A FrameAllocator that always returns `None`.
// pub struct EmptyFrameAllocator;

// unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
//     fn allocate_frame(&mut self) -> Option<PhysFrame> {
//         None
//     }
// }