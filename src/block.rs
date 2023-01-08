use std::mem;
use std::ptr::{addr_of_mut, null_mut};
use std::alloc::Layout;
use std::alloc;

use crate::index::Index;

#[derive(Debug, PartialEq)]
pub(crate) struct Chunk {
    block: *mut Block,
    start: *mut u8,
    capacity: usize,
}

impl Chunk {
    pub fn new(block: *mut Block, start: *mut u8, capacity: usize) -> Self {
        assert!(!block.is_null());

        Chunk {
            block,
            start,
            capacity,
        }
    }

    pub fn block(&self) -> *mut Block {
        self.block
    }

    pub fn start(&self) -> *mut u8 {
        self.start
    }

    pub fn copy_start(&mut self, other: &Chunk) {
        self.start = other.start;
    }

    pub fn capacity(&self) -> usize{
        self.capacity
    }

    pub fn add_capacity(&mut self, capacity: usize) {
        self.capacity += capacity;
    }

    pub fn is_next(&self, other: &Chunk) -> bool {
        unsafe {
            self.start.add(self.capacity) == other.start
        }
    }

    pub fn is_can_place<T>(&self) -> bool {
        let end = unsafe { self.start.add(self.capacity) };
        let type_offset = self.start.align_offset(mem::align_of::<T>());
        let type_end = unsafe { self.start.add(type_offset + mem::size_of::<T>()) };
        type_end <= end
    }
}

pub(crate) struct Block {
    prev: *mut Block,
    index: *mut Index,
    counter: usize,
    capacity: usize,
}

impl Block {
    pub fn prev(&self) -> *mut Block {
        self.prev
    }

    pub fn has_index(&self) -> bool {
        assert!(!self.index.is_null());
        !self.index.is_null()
    }

    pub fn reset_index(&mut self) {
        assert!(!self.index.is_null());
        self.index = null_mut();
    }

    pub fn counter(&self) -> usize {
        self.counter
    }

    pub fn increment_counter(&mut self) {
        self.counter += 1;
    }

    pub fn decrement_counter(&mut self) {
        assert!(self.counter > 0);
        self.counter -= 1;
    }

    pub fn merge_insert_free_chunk_to_index(&self, chunk: Chunk) {
        assert!(!self.index.is_null());
        unsafe{&mut (*self.index)}.merge_insert_free_chunk(chunk);
    }

    // pub fn capacity(&self) -> usize {
    //     self.capacity
    // }

    pub fn alloc_block(prev: *mut Block, index: *mut Index, capacity: usize) -> (*mut Block, Chunk) {
        assert!(capacity != 0);
        let Ok(layout) = Layout::array::<u8>(mem::size_of::<Block>() + capacity) else {
            panic!("capacity overflow")
        };

        let Ok(layout) = layout.align_to(mem::align_of::<Block>()) else {
            panic!("align error")
        };

        unsafe {
            let block: *mut Block = alloc::alloc(layout).cast();
            assert_eq!(block.align_offset(mem::align_of::<Block>()), 0);
            block.write(Block {prev, index, counter: 0, capacity});
            (block, Chunk::new(block, block.add(1).cast(), capacity))
        }
    }

    pub fn drop_block(block: *mut Block) {
        assert_eq!(unsafe{&(*block)}.counter, 0);
        let Ok(layout) = Layout::array::<u8>(mem::size_of::<Block>() + unsafe{&(*block)}.capacity) else {
            todo!()
        };

        //#TODO is it necessary?
        let Ok(layout) = layout.align_to(mem::align_of::<Block>()) else {
            todo!()
        };

        unsafe {
            block.drop_in_place();
            alloc::dealloc(block.cast(), layout);
        }
    }
}