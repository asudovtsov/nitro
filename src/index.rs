use std::mem;use std::alloc::Layout;
use std::alloc;

use crate::common::UnsafeArray;
use crate::common::UnsafeTable;

type PartType = usize;

pub(crate) const BITS_PER_BYTE: usize = 8;
pub(crate) const BYTES_PER_PART: usize = mem::size_of::<PartType>();
pub(crate) const BITS_PER_PART: usize = BITS_PER_BYTE * BYTES_PER_PART;

pub(crate) struct AlignedMask {
    data: *mut PartType,
    part_count: usize,
}

impl AlignedMask {
    pub fn new(part_count: usize) -> Self {
        let Ok(layout) = Layout::array::<PartType>(part_count) else {
            todo!()
        };

        AlignedMask {
            data: unsafe { alloc::alloc_zeroed(layout).cast() },
            part_count
        }
    }

    pub unsafe fn set(&mut self, bit_index: usize) {
        let (part, inpart_index) = self.part_at_mut(bit_index);
        *part |= 1usize << inpart_index;
    }

    pub unsafe fn reset(&mut self, bit_index: usize) {
        let (part, inpart_index) = self.part_at_mut(bit_index);
        *part &= !(1usize << inpart_index);
    }

    pub unsafe fn next_one(&self, bit_index: usize) -> Option<usize> {
        assert!(bit_index < self.part_count * BITS_PER_PART);
        let part_index = bit_index / BITS_PER_PART;
        let inpart_index = bit_index.wrapping_rem(BITS_PER_PART);
        let mut zeros = ((*self.data.add(part_index)) >> (inpart_index + 1)).trailing_zeros() as usize;
        if zeros != BITS_PER_PART {
            return Some(zeros);
        }

        for i in (part_index+1)..self.part_count {
            let part_zeros = (*self.data.add(i)).trailing_zeros() as usize;
            zeros += part_zeros;
            if part_zeros != BITS_PER_PART {
                return Some(zeros);
            }
        }
        None
    }

    unsafe fn part_at_mut(&mut self, bit_index: usize) -> (&mut usize, usize) {
        assert!(bit_index < self.part_count * BITS_PER_PART);
        let part_index = bit_index / BITS_PER_PART;
        let inpart_index = bit_index.wrapping_rem(BITS_PER_PART);
        (&mut *self.data.add(part_index), inpart_index)
    }
}

#[derive(Clone)]
pub(crate) struct Chunk {
    addr: *mut u8,
    next: Option<Box<Chunk>>,
    capacity: usize,
}

impl Chunk {
    pub fn new(addr: *mut u8, next: Option<Box<Chunk>>, capacity: usize,) -> Self {
        Chunk {
            addr,
            next,
            capacity
        }
    }

    pub fn is_following(&self, other: &Chunk) -> bool {
        if self.addr.is_null() || other.addr.is_null() {
            return false;
        }
        unsafe { self.addr.add(self.capacity) == other.addr }
    }

    pub fn is_can_place<T>(&self) -> bool {
        let end = unsafe { self.addr.add(self.capacity) };
        let type_offset = self.addr.align_offset(mem::align_of::<T>());
        let type_end = unsafe { self.addr.add(type_offset + mem::size_of::<T>()) };
        type_end <= end
    }
}

//#TODO
trait FlatCapTake {
    fn flat_take(&mut self, index: &mut FlatCapPlcIndex, capacity: usize) -> Option<Box<Chunk>>;
}

struct FlatCapEc {} // starts from EXACT capacity, increment CHUNK
struct FlatCapEi {} // starts from EXACT capacity, increment capacity INDEX
struct FlatCapMc {} // starts from MAX available capacity, decrement CHUNK
struct FlatCapMi {} // starts from MAX available capacity, decrement INDEX
struct FlatCapAc {} // starts from ALIGNED capacity, decrement CHUNK
struct FlatCapAi {} // starts from ALIGNED capacity, decrement INDEX

impl FlatCapTake for FlatCapEc {
    fn flat_take(&mut self, index: &mut FlatCapPlcIndex, capacity: usize) -> Option<Box<Chunk>> {
        None
    }
}

type ChunkUnsafeArray = UnsafeArray<Option<Box<Chunk>>>;

pub(crate) struct FlatCapPlcIndex {
    data: ChunkUnsafeArray,
    mask: AlignedMask,
    chunk_count: usize,
    flat_take: Box<dyn FlatCapTake>,
}

impl FlatCapPlcIndex {
    pub fn alloc_index(chunk_count: usize, flat_take: Box<dyn FlatCapTake>) -> *mut FlatCapPlcIndex {
        //#TODO replace with usize::div_ceil
        let mask_part_count = chunk_count / BYTES_PER_PART + usize::from(chunk_count.wrapping_rem(BYTES_PER_PART) != 0);

        let data = ChunkUnsafeArray::from(chunk_count, None);
        let mask = AlignedMask::new(mask_part_count);
        let layout = Layout::new::<FlatCapPlcIndex>();
        unsafe {
            let index: *mut FlatCapPlcIndex = alloc::alloc(layout).cast();
            assert_eq!(index.align_offset(mem::align_of::<FlatCapPlcIndex>()), 0);
            index.write(FlatCapPlcIndex{ data, mask, chunk_count, flat_take });
            index
        }
    }

    pub fn drop_index(index: *mut FlatCapPlcIndex) {
        assert!(!index.is_null());
        let layout = Layout::new::<FlatCapPlcIndex>();
        unsafe {
            ChunkUnsafeArray::drop_array(&mut (*index).data, (*index).chunk_count);
            index.drop_in_place();
            alloc::dealloc(index.cast(), layout);
        }
    }

    pub unsafe fn insert_chunk(&mut self, index: usize, chunk: Option<Box<Chunk>>) {
        let mut r#box = chunk.expect("can't add None as chunk");
        assert_eq!(index, r#box.capacity);

        let index = r#box.capacity;
        let current = self.data.replace(index, None);
        r#box.next = current;
        self.data.set(index, Some(r#box));
    }

    pub unsafe fn take_chunk_for<T>(&mut self) -> Option<Box<Chunk>> {
        let mut capacity = mem::size_of::<T>();
        assert!(capacity < self.chunk_count);

        loop {
            if let Some(r#box) = self.data.index(capacity) {
                if r#box.is_can_place::<T>() {
                    let mut chunk = self.data.replace(capacity, None).unwrap();
                    self.data.set(capacity, std::mem::take(&mut chunk.next));
                    return Some(chunk);
                }
            }

            capacity = match unsafe { self.mask.next_one(capacity) } {
                Some(bigger_capacity) => bigger_capacity,
                None => return None
            }
        }
    }

    unsafe fn for_each_in_list<F>(from: &mut Option<Box<Chunk>>, f: F) where F: Fn(&mut Option<Box<Chunk>>) {
        let mut current = from;
        while current.is_some() {
            f(&mut current);
            current = &mut current.as_mut().unwrap().next;
        }
    }

    unsafe fn chunk_for_place_from_list<T>(from: &mut Option<Box<Chunk>>) -> Option<&mut Option<Box<Chunk>>> {


        // Self::for_each_in_list(from, |chunk| { if chunk.is_can_place::<T>() {
        //     return std::mem::take(chunk);
        // }});

        // let mut current = &first;
        // loop {
        //     if let Some(ref chunk) = current {
        //         if chunk.is_can_place::<T>() {
        //             return current;
        //         }

        //         current = chunk.next;
        //     }

        //     break;
        // }
        // while current.is_some() {
        //     current = current.unwrap().next.clone();
        // }
        None
    }
}

// align
//  |
//  |   capacity --------------->
//  | 1 |0....................63|
//  | 2 |0....................63|
//  | 4 |0....................63|
//  V 8 |0....................63|
pub(crate) struct TableCapAlignPlcIndex {}

// (block_index).div_rem(index_row_count)
//  |
//  |   chunk_index ------------>
//  | 0 |0....................63|
//  | 1 |0....................63|
//  | . |0....................63|
//  V n |0....................63|

pub(crate) struct TableAddrMrgIndex {}

pub(crate) struct Matrix4dCapAlignAddrPlcMrgIndex {} // TableCapAlignPlcIndex + TableAddrMrgIndex

// pub(crate) struct FlatMergeIndex {
//     flat_index: FlatIndex,
//     ptr: ...
// }

// type ChunkUnsafeTable = UnsafeTable<Option<Box<Chunk>>>;

// struct TableMergeIndex {
//     flat_index: FlatIndex,
//     addr_table: ChunkUnsafeTable,
//     row_count: usize,
// }

// impl TableMergeIndex {
//     pub fn alloc_index(chunk_count: usize) -> *mut FlatIndex {
//         //#TODO replace with usize::div_ceil
//         let mask_part_count = chunk_count / BYTES_PER_PART + usize::from(chunk_count.wrapping_rem(BYTES_PER_PART) != 0);

//         let data = ChunkUnsafeArray::from(chunk_count, None);
//         let mask = AlignedMask::new(mask_part_count);
//         let layout = Layout::new::<FlatIndex>();
//         unsafe {
//             let index: *mut FlatIndex = alloc::alloc(layout).cast();
//             assert_eq!(index.align_offset(mem::align_of::<FlatIndex>()), 0);
//             index.write(FlatIndex{ data, mask, chunk_count });
//             index
//         }
//     }

//     pub fn drop_index(index: *mut FlatIndex) {
//         assert!(!index.is_null());
//         let layout = Layout::new::<FlatIndex>();
//         unsafe {
//             ChunkUnsafeArray::drop_array(&mut (*index).data, (*index).chunk_count);
//             index.drop_in_place();
//             alloc::dealloc(index.cast(), layout);
//         }
//     }

//     // pub unsafe fn insert_chunk(&mut self, index: usize, chunk: Option<Box<Chunk>>) {
//     //     let mut r#box = chunk.expect("can't add None as chunk");
//     //     assert_eq!(index, r#box.capacity);

//     //     let index = r#box.capacity;
//     //     let current = self.data.replace(index, None);
//     //     r#box.next = current;
//     //     self.data.set(index, Some(r#box));
//     // }

//     // pub unsafe fn take_chunk_for<T>(&mut self) -> Option<Box<Chunk>> {
//     //     let mut capacity = mem::size_of::<T>();
//     //     assert!(capacity < self.chunk_count);

//     //     loop {
//     //         if let Some(r#box) = self.data.index(capacity) {
//     //             if r#box.is_can_place::<T>() {
//     //                 let mut chunk = self.data.replace(capacity, None).unwrap();
//     //                 self.data.set(capacity, std::mem::take(&mut chunk.next));
//     //                 return Some(chunk);
//     //             }
//     //         }

//     //         capacity = match unsafe { self.mask.next_one(capacity) } {
//     //             Some(bigger_capacity) => bigger_capacity,
//     //             None => return None
//     //         }
//     //     }
//     // }
// }

// // BlockHeader {
// //     Block,
// //     block_index: usize,
// // }