use core::mem;
use core::slice;

/// On-disk file system format.

/// Block size.
pub const BSIZE: usize = 1024;

#[repr(C)]
pub struct SuperBlock {
    /// Must always be "rxv6fsmg" in little-indian.
    magic: u64,
    pub nprogs: u64,
}

#[repr(C)]
pub struct ProgBlock {
    pub nblocks: u64,
    pub start_block: u64,
}

impl SuperBlock {
    pub fn new(nprogs: u64) -> Self {
        SuperBlock {
            magic: u64::from_le_bytes(*b"rxv6fsmg"),
            nprogs,
        }
    }

    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const SuperBlock as *const u8,
                mem::size_of::<SuperBlock>(),
            )
        }
    }
}

impl ProgBlock {
    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const ProgBlock as *const u8,
                mem::size_of::<ProgBlock>(),
            )
        }
    }
}
