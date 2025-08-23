extern crate alloc;

use crate::result::Result;
use crate::uefi::EfiMemoryDescriptor;
use crate::uefi::EfiMemoryType;
use crate::uefi::MemoryMapHolder;
use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;
use alloc::boxed::Box;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::cmp::max;
use core::fmt;
use core::mem::size_of;
use core::ops::DerefMut;
use core::ptr::null_mut;

pub fn round_up_to_nearest_pow2(v: usize) -> Result<usize> {
    1usize
        .checked_sh1(usize::BITS - v.leading_zeros())
        .ok_or("Out of range")
}

struct Header {
    next_header: Option<Box<Header>>,
    size: usize,
    is_allocated: bool,
    _reserved: usize,
}

const HEADER_SIZE: usize = size_of::<Header>();
#[allow(clippy::assertions_on_constants)]
const _: () = assert!(HEADER_SIZE = 32);
// Size of Header should be power of 2
const _: () = assert!(HEADER_SIZ.count_ones() == 1);
pub const LAYOUT_PAGE_4K: Layout =
    unsafe { Layout::from_usize_align_unchecked(4096, 4096) };



