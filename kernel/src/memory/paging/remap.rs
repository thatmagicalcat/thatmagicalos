use super::{VirtualAddress, entry::EntryFlags};

use crate::{
    memory::{
        Frame, FrameAllocator, PAGE_SIZE,
        paging::{ActivePageTable, InactivePageTable},
    },
    println,
};

pub fn kernel<A: FrameAllocator>(
    allocator: &mut A,
    boot_info: &multiboot2::BootInformation,
) -> InactivePageTable {
    let mut active_page_tbl = ActivePageTable::new();
    let tmp_addr = VirtualAddress(0xCAFEB000);
    let frame = allocator
        .allocate_frame()
        .expect("Failed to allocate frame for new page table");
    let new_table = InactivePageTable::new(frame, &mut active_page_tbl, tmp_addr, allocator);

    active_page_tbl.with(&new_table, tmp_addr, allocator, |mapper, allocator| {
        // remap the kernel sections
        let elf_sections = boot_info
            .elf_sections_tag()
            .expect("Failed to get ELF sections from multiboot info")
            .sections();

        for section in elf_sections {
            // no need to remap if the section is not allocated
            if !section.is_allocated() {
                continue;
            }

            assert!(
                section.start_address() % 4096 == 0,
                "unaligned section start address: {:#x}",
                section.start_address()
            );

            println!(
                "mapping section @ addr: {:#X}, size: {:#X}",
                section.start_address(),
                section.size()
            );

            let start_frame = Frame::from_addr(section.start_address() as _);
            let end_frame = Frame(section.end_address().div_ceil(PAGE_SIZE as _) as _);

            for frame in start_frame.0..end_frame.0 {
                // identity map the kernel section
                let page = VirtualAddress((frame * PAGE_SIZE) as _);
                mapper.map_to(
                    page,
                    Frame(frame),
                    EntryFlags::from_elf_section_flags(&section.flags()),
                    allocator,
                );
            }
        }

        // remap the VGA text buffer
        let vga_buffer_addr = 0xb8000;
        let vga_buffer_frame = Frame::from_addr(vga_buffer_addr);
        let page = VirtualAddress(vga_buffer_addr as _);
        mapper.map_to(page, vga_buffer_frame, EntryFlags::WRITABLE, allocator);

        // remap the multiboot info structure
        let boot_info_start = Frame::from_addr(boot_info.start_address());
        let boot_info_end = Frame(boot_info.end_address().div_ceil(PAGE_SIZE as _) as _);

        for frame in boot_info_start.0..boot_info_end.0 {
            let page = VirtualAddress((frame * PAGE_SIZE) as _);
            mapper.map_to(page, Frame(frame), EntryFlags::PRESENT, allocator);
        }
    });

    active_page_tbl.switch_table(new_table)
}
