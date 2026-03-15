pub mod entry;
pub mod mapper;
pub mod table;

pub use entry::*;
pub use mapper::*;
pub use table::*;

use core::ops::Deref;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalAddress(pub u64);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualAddress(pub u64);

impl<T> From<T> for PhysicalAddress
where
    T: Into<u64>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl<T> From<T> for VirtualAddress
where
    T: Into<u64>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Deref for PhysicalAddress {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for VirtualAddress {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl VirtualAddress {
    pub const fn p4_idx(&self) -> usize {
        (self.0 as usize >> 39) & 0o777
    }

    pub const fn p3_idx(&self) -> usize {
        (self.0 as usize >> 30) & 0o777
    }

    pub const fn p2_idx(&self) -> usize {
        (self.0 as usize >> 21) & 0o777
    }

    pub const fn p1_idx(&self) -> usize {
        (self.0 as usize >> 12) & 0o777
    }
}
