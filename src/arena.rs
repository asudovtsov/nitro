use std::alloc::Layout;
use std::{alloc, ptr};
use std::ptr::{addr_of, addr_of_mut, null_mut};
use std::mem;
use std::rc::Rc;

use crate::block::{Chunk, Block};
use crate::index::Index;
use crate::arena_box::ArenaBox;

struct RcData<T> {
    count: usize,
    weak_count: usize,
    value: T,
}

impl<T> RcData<T> {
    fn new(value: T) -> Self {
        RcData {
            count: 1,
            weak_count: 0,
            value,
        }
    }
}

// pub struct Rc<T> {
//     data: NonNull<T>,
//     block_list: NonNull<BlockList>,
// }

// pub struct Weak<T> {
//     rc_data: NonNull<RcData<T>>,
//     block_list: NonNull<BlockList>,
// }

pub struct Arena {
    index: *mut Index,
    last: *mut Block,
    last_block_capacity: usize,
}

impl Arena {
    pub fn new() -> Self {
        Arena {
            index: null_mut(),
            last: null_mut(),
            last_block_capacity: 0,
        }
    }

    pub fn with_capacity_in_bytes(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }

        let index = Index::alloc_index();
        let (block, chunk) = Block::alloc_block(null_mut(), index, capacity);
        unsafe{&mut (*index)}.insert_free_chunk(chunk);

        Arena {
            index,
            last: block,
            last_block_capacity: capacity,
        }
    }

    pub fn free_chunk_count(&self) -> usize {
        assert!(!self.index.is_null());
        unsafe{&(*self.index)}.len()
    }

    // pub fn reserve(size: usize) {
    //     todo!();
    // }

    pub fn place_box<T>(&mut self, value: T) -> ArenaBox<T> {
        if mem::size_of::<T>() == 0 {
            todo!();
        }

        let (block, data) = self.place_internal(value);
        ArenaBox::new(block, data)
    }

    pub fn place_rc<T>(&mut self, value: T) -> Rc<T> {
        if mem::size_of::<T>() == 0 {
            todo!();
        }
        // self.place_internal(RcData::new(value));
        todo!()
    }

    pub fn defrag_free_chunks(&mut self) { todo!() }

    pub fn shrink_to_fit(&mut self) { todo!() }

    fn place_internal<T>(&mut self, value: T) -> (*mut Block, *mut T){
        assert!(!self.index.is_null());
        let chunks = unsafe{&mut (*self.index)};

        // get chunk to place data
        let index = match chunks.chunk_for_place::<T>() {
            Some(index) => index,
            None => self.grow_for::<T>()
        };

        // place data
        let chunk = chunks.chunk_at(index);
        let offset = chunk.start().align_offset(mem::align_of::<T>());
        let block = chunk.block();
        let data;
        unsafe {
            let start = chunk.start().add(offset).cast::<T>();
            start.write(value);
            data = start;
        }

        // return unoccupied mem to index
        let occupied = offset + mem::size_of::<T>();
        if occupied < chunk.capacity() {
            let start = unsafe { chunk.start().add(occupied) };
            let chunk = Chunk::new(chunk.block(), start, chunk.capacity() - occupied);
            chunks.merge_insert_free_chunk(chunk);
        }

        (block, data)
    }

    fn grow_for<T>(&mut self) -> usize {
        if self.index.is_null() {
            self.last_block_capacity = mem::size_of::<T>(); //#TODO process null sized types
            self.index = Index::alloc_index();
            let (block, chunk) = Block::alloc_block(null_mut(), self.index, self.last_block_capacity);
            self.last = block;
            return unsafe{&mut (*self.index)}.insert_free_chunk(chunk)
        }

        loop {
            self.last_block_capacity *= 2;
            let (block, chunk) = Block::alloc_block(self.last, self.index, self.last_block_capacity);
            self.last = block;
            let chunks = unsafe{&mut (*self.index)};
            let index = chunks.insert_free_chunk(chunk);
            if chunks.chunk_at(index).is_can_place::<T>() {
                break index;
            }
        }
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        if !self.index.is_null() {
            assert!(!self.last.is_null());

            let mut block_ptr = self.last;
            while !block_ptr.is_null() {
                let block = unsafe{&mut (*block_ptr)};
                block_ptr = block.prev();

                if block.counter() == 0 {
                    Block::drop_block(block_ptr);
                    continue;
                }

                block.reset_index();
            }

            Index::drop_index(self.index);
        }
    }
}