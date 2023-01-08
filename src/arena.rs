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
}

impl Arena {
    pub fn new() -> Self {
        Arena {
            index: null_mut(),
            last: null_mut(),
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

    fn place_internal<T>(&mut self, value: T) -> (*mut Block, *mut T) {
        // get chunk to place data
        let chunks;
        let index;
        if self.index.is_null() {
            index = self.grow_for::<T>();
            chunks = unsafe{&mut (*self.index)};
        } else {
            chunks = unsafe{&mut (*self.index)};
            index = match chunks.chunk_for_place::<T>() {
                Some(index) => index,
                None => self.grow_for::<T>()
            }
        }

        // place data
        let chunk = chunks.remove_chunk_at(index);
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
            //#TODO process null sized types
            self.index = Index::alloc_index();
            let (block, chunk) = Block::alloc_block(null_mut(), self.index, mem::size_of::<T>());
            self.last = block;
            return unsafe{&mut (*self.index)}.insert_free_chunk(chunk)
        }

        loop {
            let capacity = unsafe{&(*self.last)}.capacity() * 2;
            let (block, chunk) = Block::alloc_block(self.last, self.index, capacity);
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

                if block.counter() == 0 {
                    Block::drop_block(block_ptr);
                    block_ptr = block.prev();
                    continue;
                }

                block.reset_index();
                block_ptr = block.prev();
            }

            Index::drop_index(self.index);
        }
    }
}