extern crate alloc;

use crate::result::Result;
use crate::uefi::EfiMemoryDescriptor;
use crate::uefi::EfiMemoryType;
use crate::uefi::MemoryMapHolder;
use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;
//  大きなデータや可変長データをヒープ領域で管理するための型
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

    unsafe fn new_from_addr(addr: usize) -> Box<Header> {
        let header = addr as *mut Header;
        leader.write(Header {
            next_header: None,
            size: 0,
            is_allocated: false,
            _reserved: 0,
        });
        Box::from_raw(addr as *mut Header)
    }
    // Note: std::alloc::Layout doc says:
    // > すべてのレイアウトは、対応するサイズと 2 のべき乗のアライメントを持つ。
    fn provide(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        // max: 大きい方を返す
        let size = max(round_ip_up_to_nearest_pow2(size).ok()?, HEADER_SIZE);
        let align = max(align, HEADER_SIZE);
        if self.is_allocated() || !self.can_provide(size, align) {
            None
        } else {
            // 各文字は 32 バイトのチャンクを表す
            // Note: std::alloc::Layout doc says:
            // > すべてのレイアウトは、対応するサイズと 2 のべき乗のアライメントを持つ。
            //
            // |-----|----------------- self ---------|----------
            // |-----|----------------------          |----------
            //                                        ^ self.end_addr()
            //                              |-------|-
            //                              ^ header_for_allocated
            //                               ^ allocated_addr
            //                                      ^ header_for_padding
            //                                      ^
            // header_for_allocated.end_addr() によって、
            // self は要求されたオブジェクトを割り当てるのに十分な領域を持っている
            //
            // 割り当て対象のオブジェクト用のヘッダを作成する

            // 割り当てられたオブジェクト用のヘッダを作成する
            let mut size_used = 0;
            let allocated_addr = (self.end_addr() - size) & !(align - 1);
            let mut header_for_allocated = 
                unsafe { Self::new_from_addr(allocated_addr - HEADER_SIZE) };
            header_for_allocated.is_allocated = true;
            header_for_allocated.size = size + HEADER_SIZE;
            size_used += header_for_allocated.size;
            header_for_allocated.next_header = self.next_header.take();
            if header_for_allocated.end_addr() != self.end_addr() {
                // padding 用のヘッダを作成する
                let mut header_for_padding = unsafe {
                    Self::new_from_addr(header_for_allocated.end_addr())
                };
                header_for_padding.is_allocated = false;
                header_for_padding.size = 
                    self.end_addr() - header_for_allocated.end_addr();
                size_used += header_for_padding.size;
                header_for_padding.next_header = 
                    header_for_allocated.next_header.take();
                header_for_allocated.next_header = Some(header_for_padding);
            }
            // selfを縮小する
            assert!(self.size >= size_used + HEADER_SIZE);
            self.size -= size_used;
            self.next_header = Some(header_for_allocated);
            Some(allocated_addr as *mut u8)
        }
    }
}

// DropトレイトをHeaderに実装: デストラクタに相当
// Headerがドロップされるとパニックを起こすようにする
impl Drop for Header {
    fn drop(&mut self) {
        panic!("Header should not be dropped!");
    }
}

impl fmt::Dubug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "Header @ {:#018} {{ size: {:#018X}, is_allocated: {} }}",
            self as *const Header as usize,
            self.size,
            self.is_allocated()
        )
    }
}