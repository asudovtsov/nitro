use std::alloc::Layout;
use std::alloc;
use std::mem;

use crate::index::{Index64, Chunk64};

pub(crate) struct Block64 {
    prev: *mut Block64,
    index: *mut Index64,
    counter: usize,
}

impl Block64 {
    pub fn prev(&self) -> *mut Block64 {
        self.prev
    }

    // pub fn has_index(&self) -> bool {
    //     !self.index.is_null()
    // }

    // pub fn reset_index(&mut self) {
    //     assert!(!self.index.is_null());
    //     self.index = null_mut();
    // }

    pub fn alloc_block(prev: *mut Block64, index: *mut Index64) -> (*mut Block64, Chunk64) {
        let capacity = 64;
        assert!(!index.is_null());
        assert!(capacity != 0);
        let Ok(layout) = Layout::array::<u8>(mem::size_of::<Block64>() + capacity) else {
            panic!("capacity overflow")
        };

        unsafe {
            let block: *mut Block64 = alloc::alloc(layout).cast();
            assert_eq!(block.align_offset(mem::align_of::<Block64>()), 0);
            block.write(Block64 {prev, index, counter: 0});
            (block, Chunk64::new(block, block.add(1).cast(), capacity as _))
        }
    }

    pub fn drop_block(block: *mut Block64) {
        assert!(!block.is_null());
        let capacity = 64;
        let Ok(layout) = Layout::array::<u8>(mem::size_of::<Block64>() + capacity) else {
            todo!()
        };

        unsafe {
            block.drop_in_place();
            alloc::dealloc(block.cast(), layout);
        }
    }
}