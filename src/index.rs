use std::mem;
use std::alloc::Layout;
use std::alloc;

use crate::common::RawArray;
use crate::common::LinkedList;
use crate::block::Block64;
use crate::mask::Mask64;

pub(crate) struct Chunk64 {
    block: *mut Block64,
    addr: *mut u8,
    capacity: u8,
}

impl Chunk64 {
    pub fn new(block: *mut Block64, addr: *mut u8, capacity: u8) -> Self {
        Chunk64 {
            block,
            addr,
            capacity
        }
    }

    pub fn block(&self) -> &Block64 {
        unsafe {&*self.block}
    }

    pub fn is_following(&self, other: &Chunk64) -> bool {
        if self.addr.is_null() || other.addr.is_null() {
            return false;
        }
        unsafe { self.addr.add(self.capacity as _) == other.addr }
    }

    pub fn is_can_place(&self, size: usize, align: usize) -> bool {
        let end = unsafe { self.addr.add(self.capacity as _) };
        let type_offset = self.addr.align_offset(align);
        let type_end = unsafe { self.addr.add(type_offset + size) };
        type_end <= end
    }
}

type CapacityArray = RawArray<Option<Chunk64>>; //#TODO RawArray<StableStack<Chunk>>

pub(crate) struct Index64 { //#TODO bring to RawIndex by removing block_capacity
    table: CapacityArray,//#TODO RawArray<StableStack<Chunk>>
    mask: Mask64,
    block_to_capacity: Vec<Option<usize>>,
    free_list: LinkedList<Chunk64>,
    block_capacity: u8,
    drain: Option<Chunk64>
    // flat_take: Box<dyn FlatCapTake>,
}

impl Index64 {
    pub fn alloc_index(block_count: usize) -> *mut Index64 {
        let block_capacity = 64u8;
        let layout = Layout::new::<Index64>();
        unsafe {
            let index: *mut Index64 = alloc::alloc(layout).cast();
            assert_eq!(index.align_offset(mem::align_of::<Index64>()), 0);
            index.write(Index64{
                table: CapacityArray::default_filled((block_capacity - 1) as _),
                mask: Mask64::new(),
                block_to_capacity: vec![None; block_count],
                free_list: LinkedList::new(),
                block_capacity,
                drain: None, //#TODO set on first block
            });
            index
        }
    }

    pub fn drop_index(index: *mut Index64) {
        assert!(!index.is_null());
        let layout = Layout::new::<Index64>();
        unsafe {
            CapacityArray::drop_array(&mut (*index).table, ((*index).block_capacity - 1) as _);
            index.drop_in_place();
            alloc::dealloc(index.cast(), layout);
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk64) {
        assert_ne!(chunk.capacity, 0);
        assert!(chunk.capacity <= self.block_capacity);

        // set chunk as drain
        if self.drain.is_none() {
            self.drain = Some(chunk);
            return;
        }

        // push chunk of empty block in free_list
        let block_capacity = 64;
        if chunk.capacity == block_capacity {
            let block_index = chunk.block().block_index();
            if let Some(capacity) = self.map_to_capacity(block_index) {
                std::mem::take(&mut self.block_to_capacity[block_index]);
                unsafe { self.table.set(capacity, None); }
                self.mask.reset(capacity);
            }
            self.free_list.push_front(chunk);
            return;
        }

        // push chunk in table if corresponding cell is free
        if unsafe { self.table.index(chunk.capacity as _).is_none() } {
            let block_index = chunk.block().block_index();
            match self.map_to_capacity(block_index) {
                Some(capacity) if capacity < chunk.capacity as _ => {
                    self.block_to_capacity[block_index] = Some(chunk.capacity as _);
                    unsafe {
                        self.table.set(chunk.capacity as _, Some(chunk));
                        self.table.set(capacity, None);
                        self.mask.reset(capacity);
                    }
                },
                None => {
                    self.block_to_capacity[block_index] = Some(chunk.capacity as _);
                    unsafe {
                        self.table.set(chunk.capacity as _, Some(chunk));
                    }
                },
                _ => {},
            }
        }
    }

    pub fn take_chunk_for(&mut self, size: usize, align: usize) -> Option<Chunk64> {
        // try take chunk from drain
        if matches!(self.drain, Some(ref mut chunk) if chunk.is_can_place(size, align)) {
            return mem::take(&mut self.drain)
        }

        // try take chunk from table
        let mut cap_index = 63 - self.mask.trailing_zeros();
        while cap_index >= size {
            let chunk_opt = unsafe { self.table.index_mut(cap_index) };
            if matches!(chunk_opt, Some(chunk) if chunk.is_can_place(size, align)) {
                return mem::take(chunk_opt);
            }
            cap_index -= 1;
        }

        // try take chunk from free list
        self.free_list.pop_front()
    }

    fn map_to_capacity(&mut self, block_index: usize) -> Option<usize> {
        if block_index < self.block_to_capacity.len() {
            self.block_to_capacity[block_index]
        } else {
            None
        }
    }
}