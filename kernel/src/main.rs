#![no_std]
#![no_main]
#![allow(unused)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]

// SAFETY: trust me bro
unsafe extern "C" {
    safe static kernel_start: [u8; 0];
    safe static kernel_end: [u8; 0];
}

use core::{arch::asm, fmt::Write, ptr};

use multiboot2 as mb2;

use vga_buffer::{Buffer, Color, Writer};

mod interrupts;
mod macros;
mod memory;
mod vga_buffer;
mod volatile;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(multiboot_info_addr: u32) -> ! {
    interrupts::init();
    println!("Hello, World!");

    let boot_info = unsafe {
        mb2::BootInformation::load(multiboot_info_addr as *const mb2::BootInformationHeader)
    }
    .expect("Failed to load multiboot info");

    let memory_map_tag = boot_info
        .memory_map_tag()
        .expect("Memory map tag not found in multiboot info");

    println!("Memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!(
            "  - start: {:#010x}, end: {:#010x}, size: {} KB, type: {:?}",
            area.start_address(),
            area.end_address(),
            (area.end_address() - area.start_address()) / 1024,
            area.typ()
        );
    }

    let mut frame_allocator = memory::BitmapFrameAllocator::new(&boot_info);
    let frame = frame_allocator.allocate_frame().unwrap();
    let ptr = frame.get_ptr();

    println!("Total frames: {}", frame_allocator.total_frames);

    unsafe { *ptr = 42 };
    println!("Wrote value {} to address {:p}", unsafe { *ptr }, ptr);
    frame_allocator.deallocate_frame(frame);

    #[allow(clippy::empty_loop)]
    loop {}
}

/// Returns the start and end addresses of the kernel in memory.
pub fn kernel_bounds() -> (usize, usize) {
    unsafe {
        (
            &kernel_start as *const _ as usize,
            &kernel_end as *const _ as usize,
        )
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    vga_buffer::WRITER
        .lock()
        .set_color(Color::LightRed, Color::Black);
    println!("Panic: {}", info);

    loop {}
}
