use std::ptr::null_mut;
use std::mem;
use std::rc::Rc;

use crate::block::Block64;
use crate::index::Index64;
// use crate::arena_box::ArenaBox;

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

pub enum DefragLevel {
    L0,
    L1,
    L2,
    L3,
    L4
}

pub struct Builder64 {
    block_count: usize,
    grow_factor: usize,
    index_grow_factor: usize,
    defrag_level: DefragLevel,
}

impl Builder64 {
    fn new() -> Self {
        Builder64 {
            block_count: 0,
            grow_factor: 1,
            index_grow_factor: 0,
            defrag_level: DefragLevel::L4,
        }
    }

    fn block_count(mut self, count: usize) -> Self {
        self.block_count = count;
        self
    }

    fn grow_factor(mut self, factor: usize) -> Self {
        self.grow_factor = factor;
        self
    }

    fn index_grow_factor(mut self, factor: usize) -> Self {
        self.grow_factor = factor;
        self
    }

    fn defrag_level(mut self, level: DefragLevel) -> Self {
        self.defrag_level = level;
        self
    }

    fn build(self) -> Arena64 {
        let mut arena = Arena64 {
            index: null_mut(),
            last: null_mut(),
            grow_factor: self.grow_factor,
            index_grow_factor: self.index_grow_factor,
            defrag_level: self.defrag_level,
        };
        arena.grow(self.block_count);
        arena
    }
}

pub struct Arena64 {
    index: *mut Index64,
    last: *mut Block64,
    grow_factor: usize,
    index_grow_factor: usize,
    defrag_level: DefragLevel,
}

impl Arena64 {
    pub fn new() -> Self {
        Arena64 {
            index: null_mut(),
            last: null_mut(),
            grow_factor: 1,
            index_grow_factor: 0,
            defrag_level: DefragLevel::L4,
        }
    }

    pub fn with() -> Builder64 {
        Builder64::new()
    }

    pub fn block_count(&self) -> usize {
        let mut block = self.last;
        let mut count = 0;
        while !block.is_null() {
            count += 1;
            block = unsafe{&(*block)}.prev();
        }
        count
    }

    // pub fn free_chunk_count(&self) -> usize {
    //     assert!(!self.index.is_null());
    //     unsafe{&(*self.index)}.len()
    // }

    //#TODO try_grow
    pub fn grow(&mut self, block_count: usize) {
        if block_count == 0 {
            return;
        }

        if self.grow_factor == 0 {
            panic!("can't grow cause grow factor is 0")
        }

        if self.index.is_null() {
            self.index = Index64::alloc_index(block_count);
        }

        for i in 0..block_count {
            let (block, chunk) = Block64::alloc_block(i, self.last, self.index);
            unsafe { (*self.index).add_chunk(chunk); }
            self.last = block;
        }

        //#TODO expand index.block_to_capacity
    }

    // pub fn place_box<T>(&mut self, value: T) -> ArenaBox<T> {
    //     if mem::size_of::<T>() == 0 {
    //         todo!();
    //     }

    //     let (block, data) = self.place_internal(value);
    //     ArenaBox::new(block, data)
    // }

    // pub fn place_rc<T>(&mut self, value: T) -> Rc<T> {
    //     if mem::size_of::<T>() == 0 {
    //         todo!();
    //     }
    //     // self.place_internal(RcData::new(value));
    //     todo!()
    // }

    // pub fn defrag_free_chunks(&mut self) { todo!() }

    // pub fn shrink_to_fit(&mut self) { todo!() }

    fn place_internal<T>(&mut self, value: T) -> (*mut Block64, *mut T) {
        // get chunk to place data

        todo!()
        // let chunks;
        // let index;
        // if self.index.is_null() {
        //     index = self.grow_for::<T>();
        //     chunks = unsafe{&mut (*self.index)};
        // } else {
        //     chunks = unsafe{&mut (*self.index)};
        //     index = match chunks.chunk_for_place::<T>() {
        //         Some(index) => index,
        //         None => self.grow_for::<T>()
        //     }
        // }

        // // place data
        // let chunk = chunks.remove_chunk_at(index);
        // let offset = chunk.start().align_offset(mem::align_of::<T>());
        // let block = chunk.block();
        // let data;
        // unsafe {
        //     let start = chunk.start().add(offset).cast::<T>();
        //     start.write(value);
        //     data = start;
        // }

        // // return unoccupied mem to index
        // let occupied = offset + mem::size_of::<T>();
        // if occupied < chunk.capacity() {
        //     let start = unsafe { chunk.start().add(occupied) };
        //     let chunk = Chunk::new(chunk.block(), start, chunk.capacity() - occupied);
        //     chunks.merge_insert_free_chunk(chunk);
        // }

        // (block, data)
    }
}

// impl Drop for Arena {
//     fn drop(&mut self) {
//         if !self.index.is_null() {
//             assert!(!self.last.is_null());

//             let mut block_ptr = self.last;
//             while !block_ptr.is_null() {
//                 let block = unsafe{&mut (*block_ptr)};

//                 // println!("# Arena::drop block {:?} prev {:?}", block_ptr, block.prev());
//                 if block.counter() == 0 {
//                     let prev = block.prev();
//                     Block::drop_block(block_ptr);
//                     block_ptr = prev;
//                     continue;
//                 }

//                 block.reset_index();
//                 block_ptr = block.prev();
//             }

//             Index::drop_index(self.index);
//         }
//     }
// }