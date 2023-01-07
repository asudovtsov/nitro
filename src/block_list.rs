use std::ptr::{NonNull, addr_of_mut};
use std::mem;

#[derive(Debug, PartialEq)]
pub(crate) struct Chunk {
    block: NonNull<Block>,
    start: NonNull<u8>,
    capacity: usize,
}

impl Chunk {
    pub fn new(block: NonNull<Block>, start: *mut u8, capacity: usize) -> Self {
        Chunk {
            block,
            start: unsafe { NonNull::new_unchecked(start) },
            capacity,
        }
    }

    pub fn block(&self) -> NonNull<Block> {
        self.block
    }

    pub fn start(&self) -> &NonNull<u8> {
        &self.start
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
            self.start.as_ptr().add(self.capacity) == other.start.as_ptr()
        }
    }

    pub fn is_can_place<T>(&self) -> bool {
        let start = self.start.as_ptr();
        let end = unsafe { start.add(self.capacity) };
        let type_offset = start.align_offset(mem::align_of::<T>());
        let type_end = unsafe { start.add(type_offset + mem::size_of::<T>()) };
        type_end <= end
    }
}

pub(crate) struct Block {
    pub block_list: NonNull<BlockList>,
    pub previous: *mut Block,
    pub counter: usize,
}

impl Block {
    // fn new(capacity: usize) -> Self {
    //     let mut header = BlockHeader { next: NonNull::dangling(), capacity };
    //     header.next = unsafe { NonNull::new_unchecked(addr_of_mut!(header)) };
    //     header
    // }

    // fn next(&self) -> Option<&BlockHeader> {
    //     if addr_of!(*self).ne(&self.next.as_ptr().cast_const()) {
    //         return unsafe { Some(self.next.as_ref()) }
    //     }
    //     None
    // }

    // fn set_next(&mut self, next: *mut BlockHeader) {
    //     self.next = unsafe { NonNull::new_unchecked(next) };
    // }

    // fn as_non_null(&mut self) -> NonNull<BlockHeader> {
    //     unsafe { NonNull::new_unchecked(addr_of_mut!(*self)) }
    // }

    // fn chunk_non_null(&mut self) -> NonNull<Chunk> {
    //     unsafe { NonNull::new_unchecked(addr_of_mut!(self.chunk)) }
    // }
}

pub(crate) struct BlockList {
    tail: *mut Block,
}

impl BlockList {
    pub fn new(block: NonNull<Block>) -> Self {
        let list = BlockList { tail: block.as_ptr(), };
        unsafe {
            assert!(block.as_mut().counter == 0);
            block.as_mut().counter = 1;
            block.as_mut().block_list = NonNull::new_unchecked(addr_of_mut!(list));
        }
        list
    }

    pub fn push(&mut self, mut block: NonNull<Block>) {
        unsafe {
            assert!(block.as_mut().counter == 0);
            block.as_mut().counter = 1;
            block.as_mut().block_list = NonNull::new_unchecked(addr_of_mut!(*self));
        }

        if !self.tail.is_null() {
            unsafe { block.as_mut().previous = (*self.tail).previous; }
        }

        self.tail = block.as_ptr();
    }

    pub fn drop_block(list: NonNull<BlockList>, block: NonNull<Block>) {
        todo!();
    }

    // fn tail_as_ref(&self) -> &Block {
    //     unsafe { &(*self.tail) }
    // }
}