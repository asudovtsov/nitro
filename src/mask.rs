pub trait Mask {
    type IndexType;

    fn new() -> Self;
    fn trailing_zeros(&self) -> Self::IndexType;
    fn leading_zeros(&self) -> Self::IndexType;
    fn unclamped_trailing_zeros(&self, bit_index: Self::IndexType) -> Self::IndexType;
    fn unclamped_leading_zeros(&self, bit_index: Self::IndexType) -> Self::IndexType;
    fn set(&mut self, bit_index: Self::IndexType);
    fn reset(&mut self, bit_index: Self::IndexType);
}

// type PartType = usize;

// pub(crate) const BITS_PER_BYTE: usize = 8;
// pub(crate) const BYTES_PER_PART: usize = mem::size_of::<PartType>();
// pub(crate) const BITS_PER_PART: usize = BITS_PER_BYTE * BYTES_PER_PART;

// pub(crate) struct AlignedMask {
//     data: *mut PartType,
//     part_count: usize,
// }

// impl AlignedMask {
//     pub fn new(part_count: usize) -> Self {
//         let Ok(layout) = Layout::array::<PartType>(part_count) else {
//             todo!()
//         };

//         AlignedMask {
//             data: unsafe { alloc::alloc_zeroed(layout).cast() },
//             part_count
//         }
//     }

//     pub unsafe fn set(&mut self, bit_index: usize) {
//         let (part, inpart_index) = self.part_at_mut(bit_index);
//         *part |= 1usize << inpart_index;
//     }

//     pub unsafe fn reset(&mut self, bit_index: usize) {
//         let (part, inpart_index) = self.part_at_mut(bit_index);
//         *part &= !(1usize << inpart_index);
//     }

//     pub unsafe fn next_one(&self, bit_index: usize) -> Option<usize> {
//         if bit_index >= self.part_count * BITS_PER_PART {
//             return None;
//         }

//         let part_index = bit_index / BITS_PER_PART;
//         let inpart_index = bit_index.wrapping_rem(BITS_PER_PART);
//         let mut zeros = ((*self.data.add(part_index)) >> (inpart_index + 1)).trailing_zeros() as usize;
//         if zeros != BITS_PER_PART {
//             return Some(zeros);
//         }

//         for i in (part_index+1)..self.part_count {
//             let part_zeros = (*self.data.add(i)).trailing_zeros() as usize;
//             zeros += part_zeros;
//             if part_zeros != BITS_PER_PART {
//                 return Some(zeros);
//             }
//         }
//         None
//     }

//     unsafe fn part_at_mut(&mut self, bit_index: usize) -> (&mut usize, usize) {
//         assert!(bit_index < self.part_count * BITS_PER_PART);
//         let part_index = bit_index / BITS_PER_PART;
//         let inpart_index = bit_index.wrapping_rem(BITS_PER_PART);
//         (&mut *self.data.add(part_index), inpart_index)
//     }
// }

macro_rules! fixed_mask_impl {
    ($Mask:ty, $size:tt, $DataType:ty, $IndexType:ty) => {
        impl Mask for $Mask {
            type IndexType = $IndexType;
            fn new() -> Self {
                Self { data: 0 }
            }

            fn trailing_zeros(&self) -> $IndexType {
                self.data.trailing_zeros() as _
            }

            fn leading_zeros(&self) -> $IndexType {
                self.data.leading_zeros() as _
            }

            fn unclamped_trailing_zeros(&self, bit_index: $IndexType) -> $IndexType {
                assert!(bit_index < $size);
                (self.data << ($size - bit_index)).leading_zeros() as _
            }

            fn unclamped_leading_zeros(&self, bit_index: $IndexType) -> $IndexType {
                assert!(bit_index < $size);
                (self.data >> (bit_index + 1)).trailing_zeros() as _
            }

            fn set(&mut self, bit_index: $IndexType) {
                assert!(bit_index < $size);
                self.data |= (1 as $DataType) << bit_index;
            }

            fn reset(&mut self, bit_index: $IndexType) {
                assert!(bit_index < $size);
                self.data &= !((1 as $DataType) << bit_index);
            }
        }
    }
}

pub(crate) struct Mask8 {
    data: u8,
}

pub(crate) struct Mask16 {
    data: u16,
}

pub(crate) struct Mask32 {
    data: u32,
}

pub(crate) struct Mask64 {
    data: u64,
}

fixed_mask_impl!{Mask8, 8, u8, u8}
fixed_mask_impl!{Mask16, 16, u16, u8}
fixed_mask_impl!{Mask32, 32, u32, u8}
fixed_mask_impl!{Mask64, 64, u64, u8}

pub(crate) struct MaskVec64 {
    data: Vec<u64>,
}

impl Mask for MaskVec64 {
    type IndexType = u32;

    fn new() -> Self {
        MaskVec64 {
            data: vec![]
        }
    }

    fn trailing_zeros(&self) -> Self::IndexType {
        todo!()
    }

    fn leading_zeros(&self) -> Self::IndexType {
        todo!()
    }

    fn unclamped_trailing_zeros(&self, bit_index: Self::IndexType) -> Self::IndexType {
        todo!()
    }

    fn unclamped_leading_zeros(&self, bit_index: Self::IndexType) -> Self::IndexType {
        todo!()
    }

    fn set(&mut self, bit_index: Self::IndexType) {
        todo!()
    }

    fn reset(&mut self, bit_index: Self::IndexType) {
        todo!()
    }
}