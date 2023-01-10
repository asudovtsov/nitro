use std::mem;use std::alloc::Layout;
use std::alloc;
use std::ptr::{addr_of_mut, null_mut};

// use crate::block::Chunk;

// pub(crate) struct Index {
//     free_chunks: Vec<Chunk>
// }

// impl Index {
//     pub fn len(&self) -> usize {
//         self.free_chunks.len()
//     }

//     pub fn chunk_at(&self, index: usize) -> &Chunk {
//         &self.free_chunks[index]
//     }

//     pub fn remove_chunk_at(&mut self, index: usize) -> Chunk {
//         self.free_chunks.remove(index)
//     }

//     // finding a chunk with sufficient capacity using a lower bound algorithm
//     pub fn lower_bound_free_capacity(&self, capacity: usize) -> Option<usize> {
//         let mut left = 0;
//         let mut len = self.free_chunks.len();
//         let mut index;
//         let mut mid;

//         while len > 0 {
//             index = left;
//             mid = len / 2;

//             index += mid;
//             if self.free_chunks[index].capacity() < capacity {
//                 if index == self.free_chunks.len() - 1 {
//                     return None;
//                 }

//                 left = index + 1;
//                 len -= mid + 1;
//                 continue;
//             }
//             len = mid;
//         }
//         Some(left)
//     }

//     pub fn chunk_for_place<T>(&self) -> Option<usize> {
//         if let Some(chunk) = self.free_chunks.last() {
//             if chunk.is_can_place::<T>() {
//                 return Some(self.free_chunks.len() - 1);
//             }
//         }

//         None

//         // let size = mem::size_of::<T>();
//         // if let Some(bound) = self.lower_bound_free_capacity(size) {
//         //     for i in bound..self.free_chunks.len() {
//         //         if self.free_chunks[i].is_can_place::<T>() {
//         //             return Some(i);
//         //         }
//         //     }
//         // }
//         // None
//     }

//     pub fn try_merge_chunks(exists_chunk: &mut Chunk, new_chunk: &Chunk) -> bool {
//         if exists_chunk.is_next(new_chunk) {
//             exists_chunk.add_capacity(new_chunk.capacity());
//             return true;
//         }
//         if new_chunk.is_next(exists_chunk) {
//             exists_chunk.copy_start(new_chunk);
//             exists_chunk.add_capacity(new_chunk.capacity());
//             return true;
//         }
//         false
//     }

//     // binary search for new chunk position
//     pub fn insert_free_chunk(&mut self, chunk: Chunk) -> usize {
//         if self.free_chunks.is_empty() {
//             self.free_chunks.push(chunk);
//             return 0;
//         }

//         let min = self.free_chunks.first().unwrap().capacity();
//         if chunk.capacity() < min {
//             self.free_chunks.insert(0, chunk);
//             return 0;
//         }

//         let max = self.free_chunks.last().unwrap().capacity();
//         if chunk.capacity() > max {
//             let index = self.free_chunks.len();
//             self.free_chunks.insert(index, chunk);
//             return index;
//         }

//         let index = (chunk.capacity() - min) / (max - min) * self.free_chunks.len();
//         self.free_chunks.insert(index, chunk);
//         index


//         // let mut left = 0;
//         // let mut right = self.len() - 1;
//         // let mut mid = (right - left) / 2;
//         // let mut index = mid;

//         // // println!("!0");
//         // loop {
//         //     let capacity = self.free_chunks[index].capacity();
//         //     // println!("!1");
//         //     if capacity == chunk.capacity() {
//         //         self.free_chunks.insert(index, chunk);
//         //         return index;
//         //     }

//         //     if capacity < chunk.capacity() {
//         //         left = index.saturating_add(1);
//         //         mid = (right - left + 1) / 2;
//         //         index += mid;
//         //     } else {
//         //         right = index.saturating_sub(1);
//         //         mid = (right - left + 1) / 2;
//         //         index -= mid;
//         //     }

//         //     if mid == 0 {
//         //         index = usize::clamp(0, mid, self.free_chunks.len());
//         //         self.free_chunks.insert(index, chunk);
//         //         return index;
//         //     }
//         // }
//     }

//     // binary search for new chunk position
//     // and attempt to merge with existing chunk on each iteration
//     pub fn merge_insert_free_chunk(&mut self, mut chunk: Chunk) -> usize {
//         self.insert_free_chunk(chunk)
//         // if self.free_chunks.is_empty() {
//         //     self.free_chunks.push(chunk);
//         //     return 0;
//         // }

//         // // let mut left = 0;
//         // // let mut right = self.len() - 1;
//         // // let mut mid = (right - left) / 2;
//         // // let mut index = mid;

//         // // // println!("!0");
//         // // loop {
//         // //     if Self::try_merge_chunks(&mut self.free_chunks[index], &chunk) {
//         // //         chunk = self.free_chunks.remove(index); //#TODO replace with swap
//         // //         left = 0;
//         // //         right = right.saturating_sub(1);
//         // //         mid = (right - left) / 2;
//         // //         index = mid;
//         // //     }

//         // //     let capacity = self.free_chunks[index].capacity();
//         // //     // println!("!1 {} {}", capacity, chunk.capacity());
//         // //     if capacity == chunk.capacity() {
//         // //         self.free_chunks.insert(index, chunk);
//         // //         return index;
//         // //     }

//         // //     if capacity < chunk.capacity() {
//         // //         left = index.saturating_add(1);
//         // //         mid = (right - left + 1) / 2;
//         // //         index += mid;
//         // //     } else {
//         // //         right = index.saturating_sub(1);
//         // //         mid = (right - left + 1) / 2;
//         // //         index -= mid;
//         // //     }

//         // //     if mid == 0 {
//         // //         index = usize::clamp(0, mid, self.free_chunks.len());
//         // //         self.free_chunks.insert(index, chunk);
//         // //         return index;
//         // //     }
//         // // }
//     }

//     pub fn alloc_index() -> *mut Index {
//         let layout = Layout::new::<Index>();
//         // let Ok(layout) = layout.align_to(mem::align_of::<Index>()) else {
//         //     panic!("align error")
//         // };

//         unsafe {
//             let index: *mut Index = alloc::alloc(layout).cast();
//             assert_eq!(index.align_offset(mem::align_of::<Index>()), 0);
//             index.write(Index { free_chunks: vec![] });
//             index
//         }
//     }

//     pub fn drop_index(index: *mut Index) {
//         let layout = Layout::new::<Index>();
//         // let Ok(layout) = layout.align_to(mem::align_of::<Index>()) else {
//         //     todo!()
//         // };

//         unsafe {
//             index.drop_in_place();
//             alloc::dealloc(index.cast(), layout);
//         }
//     }
// }

// pub(crate) struct Mask {
//     byte_count: usize,
//     // data: *mut u8,
// }

// impl Mask {
//     // pub fn new(size: usize) -> Self {
//     //     let rem = size.wrapping_rem(8);
//     //     let bytes_count = size / 8 + usize::from(rem != 0);
//     //     Mask {
//     //         bytes: vec![0; bytes_count],
//     //     }
//     // }

//     pub fn new(byte_count: usize) -> Self {
//         assert!(byte_count >= 8);
//         assert_eq!(byte_count.wrapping_rem(8), 0);

//         Mask {
//             byte_count,
//         }
//     }

//     pub fn byte_count(&self) -> usize {
//         self.byte_count
//     }

//     pub unsafe fn leading_zeros(&self) -> usize {
//         let bytes = unsafe { addr_of_mut!(self) };
//         assert!(!bytes.is_null());

//         let mut count = 0;
//         for i in 0..self.byte_count {
//             let byte = unsafe{*bytes.add(i)};
//             let zeroes = byte.leading_zeros();
//             if zeroes == 0 {
//                 break;
//             }
//             count += zeroes;
//         }
//         count as usize
//     }

//     pub fn trailing_zeros(&self) -> usize {
//         let bytes = unsafe { addr_of_mut!(self) };
//         assert!(!bytes.is_null());

//         let mut count = 0;
//         for i in 0..self.byte_count {
//             let byte = unsafe{*bytes.add(i)};
//             let zeroes = byte.trailing_zeros();
//             if zeroes == 0 {
//                 break;
//             }
//             count += zeroes;
//         }
//         count as usize
//     }

//     pub fn is_zeroed(&self) -> bool {
//         let bytes = unsafe { addr_of_mut!(self) };
//         assert!(!bytes.is_null());
//         for i in 0..self.byte_count {
//             let byte = unsafe{*bytes.add(i).cast::<u8>()};
//             if byte != 0 {
//                 return false;
//             }
//         }
//         true
//     }

//     pub fn next_one(&self, index: usize) {

//     }

//     pub fn set(&mut self, from_bit: usize, to_bit: usize, on: bool) {
//         assert!(from_bit <= to_bit);
//         assert!(to_bit < self.byte_count * 8);

//         let from_byte = from_bit / 8;
//         let to_byte = to_bit / 8;
//         for i in from_byte..=to_byte {
//             let from_bit = if i == from_byte { from_bit } else { 0 };
//             let to_bit = if i == to_byte { to_bit.wrapping_rem(8) } else { 7 };
//             let bit_mask = Self::bit_mask(from_bit, to_bit);
//             if on {
//                 *self.byte_at_mut(i) |= bit_mask;
//             } else {
//                 *self.byte_at_mut(i) &= !bit_mask;
//             }
//         }
//     }

//     fn byte_at_mut(&mut self, index: usize) -> &mut u8 {
//         let bytes = unsafe { addr_of_mut!(self) };
//         assert!(!bytes.is_null());
//         assert!(index < self.byte_count);
//         unsafe{*bytes.add(index).cast()}
//     }

//     fn bit_mask(from: usize, to: usize) -> u8 {
//         assert!(from <= to);
//         assert!(to < 8);

//         let bits = match to - from + 1 {
//             1 => 0b00000001,
//             2 => 0b00000011,
//             3 => 0b00000111,
//             4 => 0b00001111,
//             5 => 0b00011111,
//             6 => 0b00111111,
//             7 => 0b01111111,
//             8 => 0b11111111,
//             _ => unreachable!()
//         };

//         bits << from
//     }
// }

// pub(crate) struct Chunk {
//     pointer: *mut u8,
//     capacity: usize,
// }

// impl Chunk {
//     fn new(pointer: *mut u8, capacity: usize) -> Self {
//         Chunk {
//             pointer,
//             capacity,
//         }
//     }

//     pub fn contains(&self, other: &Chunk) -> bool {
//         if self.pointer.is_null() || other.pointer.is_null() {
//             return false;
//         }

//         assert_ne!(self.capacity, 0);
//         assert_ne!(other.capacity, 0);
//         unsafe {
//             self.pointer <= other.pointer
//             && self.pointer.add(self.capacity) >= other.pointer.add(other.capacity)
//         }
//     }

//     pub fn is_following(&self, other: &Chunk) -> bool {
//         if self.pointer.is_null() || other.pointer.is_null() {
//             return false;
//         }
//         unsafe { self.pointer.add(self.capacity) == other.pointer }
//     }
// }

// // pub(crate) struct FloatingPointer {
// //     current: Chunk,
// //     checked: *mut Block,
// // }

// pub(crate) struct Index {
//     last: Chunk,
//     // free: Chunk,
//     // floating_pointer: FloatingPointer,
// }

// impl Index {
//     pub fn alloc_index() -> *mut Index {
//         let layout = Layout::new::<Index>();
//         unsafe {
//             let index: *mut Index = alloc::alloc(layout).cast();
//             assert_eq!(index.align_offset(mem::align_of::<Index>()), 0);
//             index.write(Index { last: Chunk::new(null_mut(), 0) });
//             index
//         }
//     }

//     pub fn drop_index(index: *mut Index) {
//         let layout = Layout::new::<Index>();
//         unsafe {
//             index.drop_in_place();
//             alloc::dealloc(index.cast(), layout);
//         }
//     }
// }




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

struct ChunkUnsafeArray {
    data: *mut Option<Box<Chunk>>
}

impl ChunkUnsafeArray {
    pub fn new(size: usize) -> Self {
        let Ok(layout) = Layout::array::<Option<Box<Chunk>>>(size) else {
            todo!()
        };

        let Ok(layout) = layout.align_to(mem::align_of::<Option<Box<Chunk>>>()) else {
            panic!("align error")
        };

        unsafe {
            let data: *mut Option<Box<Chunk>> = alloc::alloc(layout).cast();
            for i in 0..size {
                data.add(i).write(None);
            }

            ChunkUnsafeArray { data }
        }
    }

    pub unsafe fn drop_array(array: &mut ChunkUnsafeArray, size: usize) {
        let Ok(layout) = Layout::array::<Option<Box<Chunk>>>(size) else {
            todo!()
        };

        //#TODO is it necessary?
        // let Ok(layout) = layout.align_to(mem::align_of::<*mut T>()) else {
        //     panic!("align error")
        // };

        array.data.drop_in_place();
        alloc::dealloc(array.data.cast(), layout);
    }

    pub unsafe fn take(&self, index: usize) -> Option<Box<Chunk>> {
        std::mem::take(&mut *self.data.add(index).cast())
    }

    pub unsafe fn set(&mut self, index: usize, value: Option<Box<Chunk>>) {
        self.data.add(index).write(value);
    }

    pub unsafe fn index(&self, index: usize) -> &Option<Box<Chunk>> {
        &*self.data.add(index).cast()
    }

    pub unsafe fn index_mut(&mut self, index: usize) -> &mut Option<Box<Chunk>> {
        &mut *self.data.add(index).cast()
    }
}

// impl<T: Copy> ChunkUnsafeArray<T> {
//     pub unsafe fn get(&self, index: usize) -> T {
//         *self.data.add(index).cast()
//     }
// }

pub(crate) struct Index {
    data: ChunkUnsafeArray,
    mask: AlignedMask,
    chunk_count: usize
}

impl Index {
    pub fn alloc_index(chunk_count: usize) -> *mut Index {
        //#TODO replace with usize::div_ceil
        let mask_part_count = chunk_count / BYTES_PER_PART + usize::from(chunk_count.wrapping_rem(BYTES_PER_PART) != 0);

        let data = ChunkUnsafeArray::new(chunk_count);
        let mask = AlignedMask::new(mask_part_count);
        let layout = Layout::new::<Index>();
        unsafe {
            let index: *mut Index = alloc::alloc(layout).cast();
            assert_eq!(index.align_offset(mem::align_of::<Index>()), 0);
            index.write(Index{ data, mask, chunk_count });
            index
        }
    }

    pub fn drop_index(index: *mut Index) {
        assert!(!index.is_null());
        let layout = Layout::new::<Index>();
        unsafe {
            ChunkUnsafeArray::drop_array(&mut (*index).data, (*index).chunk_count);
            index.drop_in_place();
            alloc::dealloc(index.cast(), layout);
        }
    }

    pub unsafe fn add_chunk(&mut self, chunk: Option<Box<Chunk>>) {
        let mut r#box = chunk.expect("can't add None as chunk");
        let index = r#box.capacity;
        let current = self.data.take(index);
        r#box.next = current;
        self.data.set(index, Some(r#box));
    }

    pub unsafe fn take_chunk(&mut self, index: usize) -> Option<Box<Chunk>>{
        assert!(index < self.chunk_count);
        let current = self.data.take(index);
        if let Some(mut r#box) = current {
            self.data.set(index, std::mem::take(&mut r#box.next));
            return Some(r#box);
        }

        if let Some(bigger_capacity) = unsafe { self.mask.next_one(index) } {
            return self.take_chunk(bigger_capacity)
        }
        None
    }
}