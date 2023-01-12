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

    pub fn capacity(&self) -> u8 {
        self.capacity
    }

    pub fn block_index(&self) -> usize {
        unsafe {&*self.block}.block_index()
    }

    pub fn try_consume_next(&mut self, other: &mut Chunk64) -> bool {
        if self.is_next_neighbour(&other) {
            self.capacity += other.capacity;
            other.capacity = 0;
            return true
        }
        false
    }

    pub fn try_consume_prev(&mut self, other: &mut Chunk64) -> bool {
        if other.is_next_neighbour(&self) {
            self.addr = other.addr;
            self.capacity += other.capacity;
            other.capacity = 0;
            return true
        }
        false
    }

    pub fn is_next_neighbour(&self, other: &Chunk64) -> bool {
        if self.addr.is_null() || other.addr.is_null() {
            return false;
        }
        unsafe { self.addr.add(self.capacity as _) == other.addr }
    }

    pub fn is_block_neighbour(&self, other: &Chunk64) -> bool {
        self.block == other.block
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
    block_idx_to_table: Vec<Option<usize>>,
    empty_chunks: LinkedList<Chunk64>,
    drain: Option<Chunk64>,
    block_capacity: u8,
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
                block_idx_to_table: vec![None; block_count],
                empty_chunks: LinkedList::new(),
                drain: None,
                block_capacity,
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

    pub fn add_chunk(&mut self, mut chunk: Chunk64) {
        assert_ne!(chunk.capacity, 0);
        assert!(chunk.capacity <= self.block_capacity);

        match self.drain {
            Some(ref mut drain) => {
                if drain.try_consume_next(&mut chunk)
                    || drain.try_consume_prev(&mut chunk) {
                    return;
                }

                if drain.capacity < chunk.capacity {
                    // if self.table.index(drain.capacity).is_none() {

                    // }
                    //#TODO
                    // push drain to table free cell
                    // drain = chunk
                }
            },
            None => {
                mem::swap(&mut self.drain, &mut Some(chunk));
                return;
            },
        }

        // // set chunk as drain
        // if self.drain.is_none() {
        //     self.drain = Some(chunk);
        //     return;
        // }

        // if matches!(self.drain, Some(ref d) if d.is_from_same_block(&chunk)) {
        //     if d()
        // }

        // push chunk of empty block in empty list
        let block_capacity = 64;
        if chunk.capacity == block_capacity {
            let block_index = chunk.block_index();
            if let Some(capacity) = self.map_to_table(block_index) {
                std::mem::take(&mut self.block_idx_to_table[block_index]);
                unsafe { self.table.set(capacity, None); }
                self.mask.reset(capacity as _);
            }
            //#TODO remove chunk from drain if is the same block
            self.empty_chunks.push_front(chunk);
            return;
        }

        // push chunk in table if corresponding cell is free
        if unsafe { self.table.index(chunk.capacity as _).is_none() } {
            let block_index = chunk.block_index();
            match self.map_to_table(block_index) {
                Some(capacity) if capacity < chunk.capacity as _ => {
                    self.block_idx_to_table[block_index] = Some(chunk.capacity as _);
                    unsafe {
                        self.mask.set(chunk.capacity);
                        self.mask.reset(capacity as _);
                        self.table.set(chunk.capacity as _, Some(chunk));
                        self.table.set(capacity, None);
                    }
                },
                None => {
                    self.block_idx_to_table[block_index] = Some(chunk.capacity as _);
                    unsafe {
                        self.mask.set(chunk.capacity);
                        self.table.set(chunk.capacity as _, Some(chunk));
                    }
                },
                _ => {},
                //#TODO remove chunk from drain if is the same block
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
        while cap_index as _ >= size {
            let chunk_opt = unsafe { self.table.index_mut(cap_index as _) };
            if matches!(chunk_opt, Some(chunk) if chunk.is_can_place(size, align)) {
                return mem::take(chunk_opt);
            }
            cap_index -= 1;
        }

        // try take chunk from empty list
        self.empty_chunks.pop_front()
    }

    fn map_to_table(&mut self, block_index: usize) -> Option<usize> {
        if block_index < self.block_idx_to_table.len() {
            self.block_idx_to_table[block_index]
        } else {
            None
        }
    }

    fn upgrade_drain(&mut self, chunk: Chunk64) -> Result<(), Chunk64> {
        todo!()
    }

    fn insert_to_table(&mut self, chunk: Chunk64) -> Result<(), Chunk64> {
        todo!()
    }

    fn push_to_empty_chunks(&mut self, chunk: Chunk64) -> Result<(), Chunk64> {
        todo!()
    }
}