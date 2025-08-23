extern crate alloc;

use crate::result::Result;
use crate::uefi::EfiMemoryDescriptor;
use crate::uefi::EfiMemoryType;
use crate::uefi::MemoryMapHolder;
use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;
//  大きなデータや可変超データをヒープ領域で管理するための型
use alloc::boxed::Box;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::cmp::max;
use core::fmt;
use core::mem::size_of;
use core::ops::DerefMut;
use core::ptr::null_mut;

// ある整数vを次に大きい2のべき乗に切り上げる
pub fn round_up_to_nearest_pow2(v: usize) -> Result<usize> {
    1usize
        // leading_zeros() 整数を2進数で表した時に先頭に連続する0を返す
        .checked_shl(usize::BITS - v.leading_zeros())
        .ok_or("Out of range")
}
/// 垂直バー | は、ヘッダーを持つチャンクを表す
/// before（変更前）: |– 前 —––|–– 自分 —————
/// align（整列後）:  |––––|—––|—––|—––|—––|
/// after（変更後）:  |—————||—––|––––––––

// 単方向リンクリスト
struct Header {
    next_header: Option<Box<Header>>,
    size: usize,
    is_allocated: bool,
    _reserved: usize,
}

const HEADER_SIZE: usize = size_of::<Header>();
// 定数に対してのasserをclippyが警告するのを抑制
#[allow(clippy::assertions_on_constants)]
const _: () = assert!(HEADER_SIZE = 32);
// HEADER_SIZEが2のべき乗であることを確認
const _: () = assert!(HEADER_SIZE.count_ones() == 1);
pub const LAYOUT_PAGE_4K: Layout =
// サイズ4096バイト、アライメント4096バイト = 4KBページ
    unsafe { Layout::from_usize_align_unchecked(4096, 4096) };

impl Header {
    fn can_provide(&self, size: usize, align: useze) -> bool {
        self.size >= size + HEADER_SIZE * 2 + align;
    }

    fn is_allocated(&self) -> bool {
        self.is_allocated
    }

    fn end_addr(&self) -> usize {
        self as *const Header as usize + self.size
    }
}

