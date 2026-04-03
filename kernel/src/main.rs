#![no_std]
#![no_main]
#![warn(clippy::missing_const_for_fn)]
#![allow(clippy::empty_loop)]

extern crate alloc;

mod gdt;
mod graphics;
mod hpet;
mod interrupts;
mod io;
mod ioapic;
mod kernel;
mod macros;
mod memory;
mod scheduler;
mod task;
mod thread;
mod utils;
mod volatile;

use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker, request::*};

#[rustfmt::skip]
const MIN_LOG_LEVEL: log::LevelFilter = {
    #[cfg(log_level = "trace")] { log::LevelFilter::Trace }
    #[cfg(log_level = "debug")] { log::LevelFilter::Debug }
    #[cfg(log_level = "info")] { log::LevelFilter::Info }
    #[cfg(log_level = "warn")] { log::LevelFilter::Warn }
    #[cfg(log_level = "error")] { log::LevelFilter::Error }
};

/// The virtual address where the Linear framebuffer is mapped
const LFB_VIRT_ADDR: usize = 0xFFFF_8000_0000_0000;

const WALLPAPER_DATA: &[u8] = include_bytes!("../../wallpaper.bin");
const FONT_DATA: &[u8] = include_bytes!("../../ter-u32n.psf");

#[used]
#[unsafe(link_section = ".limine_requests_start")]
static REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static BOOTLOADER: BootloaderInfoRequest = BootloaderInfoRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static MEMMAP: MemmapRequest = MemmapRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel::init();

    assert!(BASE_REVISION.is_supported(), "Limine base revision not supported");

    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("KERNEL PANIC: {info}",);
    loop {}
}
