use bitflags::bitflags;

use crate::{
    println,
    vga_buffer::{Color, WRITER},
};

use table::*;
pub use idt::IDT;

#[macro_use]
mod macros;
mod table;
mod idt;
mod handlers;

pub fn init() {
    IDT.load();
}

bitflags! {
    #[derive(Debug)]
    struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}

