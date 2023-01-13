use std::mem;
use std::alloc::Layout;
use std::alloc;

use crate::block;
use crate::common::unsafe_array::UnsafeArray;
use crate::common::linked_list::LinkedList;
use crate::block::Block64;
use crate::common::unsafe_linked_table::UnsafeLinkedTable;
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

type CapacityArray = UnsafeArray<Option<Chunk64>>; //#TODO RawArray<StableStack<Chunk>>

const BLOCK_CAPACITY: usize = 64;

pub(crate) struct Index64 { //#TODO do not add field block_capacty, just use external variable (UnsafeIndex)
    table: UnsafeLinkedTable<Chunk64>,//#TODO RawArray<StableStack<Chunk>>
    mask: Mask64,
    map: Vec<Option<usize>>,
    empty_chunks: LinkedList<Chunk64>,
    drainable: Option<Chunk64>,
    // flat_take: Box<dyn FlatCapTake>,
}

impl Index64 {
    pub fn alloc_index(block_count: usize) -> *mut Index64 {
        let layout = Layout::new::<Index64>();
        unsafe {
            let index: *mut Index64 = alloc::alloc(layout).cast();
            assert_eq!(index.align_offset(mem::align_of::<Index64>()), 0);
            index.write(Index64{
                table: UnsafeLinkedTable::with_capacity(BLOCK_CAPACITY, block_count),
                mask: Mask64::new(),
                map: vec![None; block_count],
                empty_chunks: LinkedList::new(),
                drainable: None,
            });
            index
        }
    }

    pub fn drop_index(index: *mut Index64) {
        assert!(!index.is_null());
        let layout = Layout::new::<Index64>();
        unsafe {
            UnsafeLinkedTable::<Chunk64>::drop_table(&mut (*index).table, BLOCK_CAPACITY);
            index.drop_in_place();
            alloc::dealloc(index.cast(), layout);
        }
    }

    pub fn add_chunk(&mut self, mut chunk: Chunk64) {
        assert_ne!(chunk.capacity, 0);
        assert!(chunk.capacity <= BLOCK_CAPACITY as _);

        match self.drainable {
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
                mem::swap(&mut self.drainable, &mut Some(chunk));
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

        // // push chunk of empty block in empty list
        // if chunk.capacity == BLOCK_CAPACITY as _ {
        //     let block_index = chunk.block_index();
        //     if let Some(capacity) = self.map_to_table(block_index) {
        //         std::mem::take(&mut self.map[block_index]);
        //         unsafe { self.table.set(capacity, None); }
        //         self.mask.reset(capacity as _);
        //     }
        //     //#TODO remove chunk from drainable if is the same block
        //     self.empty_chunks.push_front(chunk);
        //     return;
        // }

        // // push chunk in table if corresponding cell is free
        // if unsafe { self.table.index(chunk.capacity as _).is_none() } {
        //     let block_index = chunk.block_index();
        //     match self.map_to_table(block_index) {
        //         Some(capacity) if capacity < chunk.capacity as _ => {
        //             self.map[block_index] = Some(chunk.capacity as _);
        //             unsafe {
        //                 self.mask.set(chunk.capacity);
        //                 self.mask.reset(capacity as _);
        //                 self.table.set(chunk.capacity as _, Some(chunk));
        //                 self.table.set(capacity, None);
        //             }
        //         },
        //         None => {
        //             self.map[block_index] = Some(chunk.capacity as _);
        //             unsafe {
        //                 self.mask.set(chunk.capacity);
        //                 self.table.set(chunk.capacity as _, Some(chunk));
        //             }
        //         },
        //         _ => {},
        //         //#TODO remove chunk from drainable if is the same block
        //     }
        // }
    }

    pub fn take_chunk_for(&mut self, size: usize, align: usize) -> Option<Chunk64> {
        todo!()
        // // try take chunk from drainable
        // if matches!(self.drainable, Some(ref mut chunk) if chunk.is_can_place(size, align)) {
        //     return mem::take(&mut self.drainable)
        // }

        // // try take chunk from table
        // let mut cap_index = 63 - self.mask.trailing_zeros();
        // while (cap_index as usize) >= size {
        //     let chunk_opt = unsafe { self.table.index_mut(cap_index as _) };
        //     if matches!(chunk_opt, Some(chunk) if chunk.is_can_place(size, align)) {
        //         return mem::take(chunk_opt);
        //     }
        //     cap_index -= 1;
        // }

        // // try take chunk from empty list
        // self.empty_chunks.pop_front()
    }

    fn map_to_table(&mut self, block_index: usize) -> Option<usize> {
        if block_index < self.map.len() {
            self.map[block_index]
        } else {
            None
        }
    }

    fn set_drainable(&mut self, chunk: Chunk64) {
        todo!()
    }

    // drainable must be checked
    fn take_from_table(&mut self, size: usize, align: usize) {
        // let mut cap_index = BLOCK_CAPACITY - self.mask.trailing_zeros() as _;
        // while cap_index >= size {
        //     // let mut cursor = self.table.
        //     let chunk_opt = unsafe { self.table.index_mut(cap_index as _) };
        //     if matches!(chunk_opt, Some(chunk) if chunk.is_can_place(size, align)) {
        //         return mem::take(chunk_opt);
        //     }
        //     cap_index -= 1;
        // }
    }

    fn insert_to_table(&mut self, chunk: Chunk64) -> Result<(), Chunk64> {
        todo!()
    }

    fn push_to_empty_chunks(&mut self, chunk: Chunk64) -> Result<(), Chunk64> {
        todo!()
    }
}