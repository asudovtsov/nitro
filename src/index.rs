use std::mem::MaybeUninit;
use std::mem;
use std::ops::Index;
use std::ptr::NonNull;
use std::alloc::Layout;
use std::alloc;

use crate::block::Block64;
use crate::common::linked_list::{List, Node, OptNode};
use crate::common::unsafe_table::UnsafeTable;
use crate::mask::{Mask, Mask64};

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

    // pub fn try_consume_next(&mut self, other: &mut Chunk64) -> bool {
    //     if self.is_next_neighbour(&other) {
    //         self.capacity += other.capacity;
    //         other.capacity = 0;
    //         return true
    //     }
    //     false
    // }

    // pub fn try_consume_prev(&mut self, other: &mut Chunk64) -> bool {
    //     if other.is_next_neighbour(&self) {
    //         self.addr = other.addr;
    //         self.capacity += other.capacity;
    //         other.capacity = 0;
    //         return true
    //     }
    //     false
    // }

    // pub fn is_next_neighbour(&self, other: &Chunk64) -> bool {
    //     if self.addr.is_null() || other.addr.is_null() {
    //         return false;
    //     }
    //     unsafe { self.addr.add(self.capacity as _) == other.addr }
    // }

    // pub fn is_block_neighbour(&self, other: &Chunk64) -> bool {
    //     self.block == other.block
    // }

    pub fn place_result(&self, size: usize, align: usize) -> Result<(Option<Self>, Option<Self>), ()> {
        let end = unsafe { self.addr.add(self.capacity as _) };
        let type_offset = self.addr.align_offset(align);
        let type_end = unsafe { self.addr.add(type_offset + size) };
        if type_end > end {
            return Err(());
        }

        let first = if type_offset != 0 {
            Some(Chunk64 {
                block: self.block,
                addr: self.addr,
                capacity: type_offset as _
            })
        } else {
            None
        };

        let second = if type_end != end {
            Some(Chunk64 {
                block: self.block,
                addr: unsafe {self.addr.add(type_offset)},
                capacity: self.capacity - size as u8 - type_offset as u8
            })
        } else {
            None
        };

        Ok((first, second))
    }

    // pub fn is_can_place(&self, size: usize, align: usize) -> bool {
    //     let end = unsafe { self.addr.add(self.capacity as _) };
    //     let type_offset = self.addr.align_offset(align);
    //     let type_end = unsafe { self.addr.add(type_offset + size) };
    //     type_end <= end
    // }
}

const BLOCK64_CAPACITY: usize = 64;

type MaybeChunk64 = MaybeUninit<Chunk64>;

pub(crate) struct Index64 { //#TODO do not add field block_capacty, just use external variable (UnsafeIndex)
    table: UnsafeTable<MaybeChunk64>,
    // drain: LinkedList<MaybeChunk64>,
    reserve: List<MaybeChunk64>,
    mask: Mask64,
}

impl Index64 {
    // pub fn alloc_index(reserve_capacity: usize) -> *mut Index64 {
    //     let mut reserve = LinkedList::new();
    //     for _ in 0..reserve_capacity {
    //         reserve.push_front(MaybeUninit::uninit());
    //     }

    //     let layout = Layout::new::<Index64>();
    //     unsafe {
    //         let index: *mut Index64 = alloc::alloc(layout).cast();
    //         assert_eq!(index.align_offset(mem::align_of::<Index64>()), 0);
    //         index.write(Index64{
    //             table: UnsafeTable::new(BLOCK64_CAPACITY),
    //             // drain: LinkedList::new(),
    //             reserve,
    //             mask: <Mask64 as Mask>::new(),
    //         });
    //         index
    //     }
    // }

    pub fn drop_index(index: *mut Index64) {
        assert!(!index.is_null());
        let layout = Layout::new::<Index64>();
        unsafe {
            UnsafeTable::<MaybeChunk64>::drop_table(&mut (*index).table, BLOCK64_CAPACITY);
            index.drop_in_place();
            alloc::dealloc(index.cast(), layout);
        }
    }

    // pub fn add_chunk(&mut self, chunk: Chunk64) {
    //     assert_ne!(chunk.capacity, 0);
    //     assert!(chunk.capacity <= BLOCK_CAPACITY as _);

    //     match std::mem::take(&mut self.drainable) {
    //         Some(drainable) => {
    //             if drainable.is_block_neighbour(&chunk) {
    //                 self.drainable = Some(if chunk.capacity > drainable.capacity {chunk} else {drainable});
    //                 return;
    //             }

    //             if chunk.capacity() > drainable.capacity() {
    //                 self.drainable = Some(chunk);
    //                 self.insert_to_table_uninvariant(drainable);
    //             } else {
    //                 self.drainable = Some(drainable);
    //                 self.insert_to_table_uninvariant(chunk);
    //             }
    //         },
    //         None => {
    //             mem::swap(&mut self.drainable, &mut Some(chunk));
    //         },
    //     }
    // }

    // pub fn take_chunk_for(&mut self, size: usize, align: usize) -> Option<Chunk64> {
    //     if matches!(self.drainable, Some(drainable) if drainable.is_can_place(size, align)) {
    //         return self.drainable
    //     }
    //     if self.updagrade_drianable_for_uninvariant(size, align) {
    //         return self.drainable
    //     }
    //     None
    // }

    // // upgrade drainable without checking it current state
    // fn updagrade_drianable_for_uninvariant(&mut self, size: usize, align: usize) -> bool {
    //     match self.take_from_table_uninvariant(size, align) { //#TODO replace take+insert with swap
    //         Some(chunk) => {
    //             if let Some(drainable) = mem::take(&mut self.drainable) {
    //                 self.insert_to_table_uninvariant(drainable);
    //             }
    //             self.drainable = Some(chunk);
    //             true
    //         },
    //         None => false
    //     }
    // }

    // take from table, update mask and map but do nothing with drainable
    fn insert_to_table_uninvariant(&mut self, chunk: Chunk64) {
        self.mask.set(chunk.capacity());
        unsafe { self.table.push_front(chunk.capacity() as _, MaybeUninit::new(chunk)); }
        //#TODO update map //#CONTINUE
        todo!()
    }

    // take from table, update mask and map but do nothing with drainable
    fn take_from_table_uninvariant(&mut self, size: usize, align: usize) -> OptNode<MaybeChunk64> {
        // let zeros = self.mask.trailing_zeros() as usize;
        // assert!(zeros <= BLOCK_CAPACITY);

        // if zeros == BLOCK_CAPACITY {
        //     return None;
        // }

        // let cap_index = BLOCK_CAPACITY - zeros;
        // let mut cursor = unsafe { self.table.cursor_mut(cap_index, BLOCK_CAPACITY) };
        // loop {
        //     match cursor.current() {
        //         Some(chunk) => {
        //             if unsafe { chunk.assume_init_mut()}.is_can_place(size, align) {
        //                 let chunk = cursor.remove_current();
        //                 if cursor.current().is_none() {
        //                     self.mask.reset(cap_index as _);
        //                 }
        //                 break chunk
        //             } else {
        //                 cursor.move_next();
        //             }
        //         },
        //         None => break None
        //     }
        // }
        //#TODO update map
        todo!()
    }

    // pub unsafe fn push_to_drain(&mut self, nn: NonNull<Node<MaybeChunk64>>) {
    //     if self.drain.is_empty() {
    //         self.drain.push_node_front(nn);
    //         return;
    //     }

    //     let mut count = 0;
    //     let mut cursor = self.drain.cursor_front_mut();
    //     let mut opt_nn = Some(nn);
    //     let mut prev;

    //     loop {
    //         //#CONTINUE
    //         match cursor.current_node() {
    //             Some(current) => {
    //                 let capacity = nn.as_ref().value_ref().assume_init_ref().capacity();
    //                 let current_capacity = current.assume_init_ref().capacity();
    //                 if current_capacity < capacity {
    //                     opt_nn = cursor.replace_current_node(std::mem::take(&mut opt_nn).unwrap())
    //                 }
    //                 prev = current;
    //             },
    //             None => {
    //                 if count < 2 {
    //                     //#TOTO list.push_node_after(prev, opt_nn.unwrap())
    //                 } else {
    //                     //#TOTO list.push_node_after(prev, opt_nn.unwrap())
    //                     //#TODO return old_node to table
    //                 }
    //             },
    //         }

    //         cursor.move_next();
    //         count += 1;
    //     }
    // }
}