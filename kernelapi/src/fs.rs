use core::mem::size_of;
use core::ptr;
use core::slice;

/// On-disk file system format.

/// Block size. Must be a multiple of 512.
pub const BSIZE: usize = 1024;

pub const SB_MAGIC: u64 = u64::from_le_bytes(*b"rxv6fsmg");

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
            magic: SB_MAGIC,
            nprogs,
        }
    }

    pub fn from_u8_slice(data: &[u8]) -> Option<Self> {
        if data.len() < size_of::<SuperBlock>() {
            None
        } else {
            let obj = unsafe { ptr::read(data.as_ptr() as *const SuperBlock) };
            if obj.magic != SB_MAGIC {
                None
            } else {
                Some(obj)
            }
        }
    }

    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const SuperBlock as *const u8,
                size_of::<SuperBlock>(),
            )
        }
    }
}

impl ProgBlock {
    pub fn from_u8_slice(data: &[u8]) -> Option<Self> {
        if data.len() < size_of::<ProgBlock>() {
            None
        } else {
            let obj = unsafe { ptr::read(data.as_ptr() as *const ProgBlock) };
            Some(obj)
        }
    }

    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const ProgBlock as *const u8,
                size_of::<ProgBlock>(),
            )
        }
    }
}
