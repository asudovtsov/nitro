use std::alloc;
use std::ptr::{NonNull, addr_of, addr_of_mut, null_mut};
use std::mem;
use std::rc::Rc;

use crate::arena_box::ArenaBox;
use crate::block_list::{Chunk, Block, BlockList};

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
    free_chunks: Vec<Chunk>,
    block_list: Option<Box<BlockList>>,
    last_block_capacity: usize,
}

impl Arena {
    pub fn new() -> Self {
        Arena {
            free_chunks: vec![],
            block_list: None,
            last_block_capacity: 0,
        }
    }

    pub fn with_capacity_in_bytes(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }

        let chunk = Self::alloc_block(capacity);
        Arena {
            free_chunks: vec![chunk],
            block_list: Some(Box::new(BlockList::new(chunk.block()))),
            last_block_capacity: capacity,
        }
    }

    pub fn free_chunks_count(&self) -> usize {
        self.free_chunks.len()
    }

    // pub fn reserve(size: usize) {
    //     todo!();
    // }

    pub fn place_box<T>(&mut self, value: T) -> ArenaBox<T> {
        if mem::size_of::<T>() == 0 {
            todo!();
        }

        ArenaBox::new(self.block_list.as_ref().unwrap().as_ref(), self.place_internal(value))
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

    fn place_internal<T>(&mut self, value: T) -> NonNull<T>{
        // get chunk to place data
        let index = match self.chunk_for_place::<T>() {
            Some(index) => index,
            None => self.grow_for::<T>()
        };

        // place data
        let chunk = &self.free_chunks[index];
        let start = chunk.start().as_ptr();
        let offset = start.align_offset(mem::align_of::<T>());
        let data;
        unsafe {
            let start = start.add(offset).cast::<T>();
            start.write(value);
            data = NonNull::new_unchecked(start);
        }

        // return unoccupied mem to index
        let occupied = offset + mem::size_of::<T>();
        if occupied < chunk.capacity() {
            let start = unsafe { start.add(occupied) };
            let chunk = Chunk::new(chunk.block(), start, chunk.capacity() - occupied);
            self.insert_free_chunk(chunk);
        }

        data
    }

    fn alloc_block(capacity: usize) -> Chunk {
        assert!(capacity != 0);
        let layout = match alloc::Layout::array::<u8>(capacity + mem::size_of::<Block>()) {
            Ok(layout) => layout,
            Err(_) => panic!("capacity overflow")
        };

        let layout = match layout.align_to(mem::align_of::<Block>()) {
            Ok(layout) => layout,
            Err(_) => panic!("align error")
        };

        unsafe {
            let block: *mut Block = alloc::alloc(layout).cast();
            assert_eq!(block.align_offset(mem::align_of::<Block>()), 0);
            block.write(Block { block_list: NonNull::dangling(), previous: null_mut(), counter: 0 });

            let chunk: *mut u8 = block.add(1).cast();
            Chunk::new(NonNull::new_unchecked(block), chunk, capacity)
        }
    }

    // finding a chunk with sufficient capacity using a lower bound algorithm
    fn lower_bound_free_capacity(&self, capacity: usize) -> Option<usize> {
        let mut left = 0;
        let mut len = self.free_chunks.len();
        let mut index;
        let mut mid;

        while len > 0 {
            index = left;
            mid = len / 2;

            index += mid;
            if self.free_chunks[index].capacity() < capacity {
                if index == self.free_chunks.len() - 1 {
                    return None;
                }

                left = index + 1;
                len -= mid + 1;
                continue;
            }
            len = mid;
        }
        Some(left)
    }

    fn chunk_for_place<T>(&self) -> Option<usize> {
        let size = mem::size_of::<T>();
        if let Some(bound) = self.lower_bound_free_capacity(size) {
            for i in bound..self.free_chunks.len() {
                if self.free_chunks[i].is_can_place::<T>() {
                    return Some(i);
                }
            }
        }
        None
    }

    fn try_merge_chunks(exists_chunk: &mut Chunk, new_chunk: &Chunk) -> bool {
        if exists_chunk.is_next(new_chunk) {
            exists_chunk.add_capacity(new_chunk.capacity());
            return true;
        }
        if new_chunk.is_next(exists_chunk) {
            exists_chunk.copy_start(new_chunk);
            exists_chunk.add_capacity(new_chunk.capacity());
            return true;
        }
        false
    }

    // binary search for new chunk position
    // and attempt to merge with existing chunk on each iteration
    fn insert_free_chunk(&mut self, mut chunk: Chunk) -> usize { //#CONTINUE
        if self.free_chunks.is_empty() {
            self.free_chunks.push(chunk);
            return 0;
        }

        let mut index = self.free_chunks.len() / 2;
        loop {
            let current = &mut self.free_chunks[index];

            if Self::try_merge_chunks(current, &chunk) {
                chunk = self.free_chunks.remove(index); //#TODO replace with swap
                index = self.free_chunks.len() / 2;
            }

            let step = index / 2;
            if current.capacity() == chunk.capacity() || step == 0 {
                self.free_chunks.insert(index + 1, chunk);
                break index + 1;
            }

            if current.capacity() < chunk.capacity() {
                index += index / 2;
            } else {
                index -= index / 2;
            }
        }
    }

    fn grow_for<T>(&mut self) -> usize {
        if self.block_list.is_none() {
            self.last_block_capacity = mem::size_of::<T>(); //#TODO process null sized types
            let chunk = Self::alloc_block(self.last_block_capacity);
            self.block_list = Some(Box::new(BlockList::new(chunk.block())));
            return self.insert_free_chunk(chunk);
        }

        loop {
            self.last_block_capacity *= 2;
            let chunk = Self::alloc_block(self.last_block_capacity);
            self.block_list.as_mut().unwrap().push(chunk.block());
            let index = self.insert_free_chunk(chunk);

            if self.free_chunks[index].is_can_place::<T>() {
                break index;
            }
        }
    }
}