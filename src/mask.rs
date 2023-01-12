use std::mem;
use std::alloc::Layout;
use std::alloc;

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
        if bit_index >= self.part_count * BITS_PER_PART {
            return None;
        }

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

pub(crate) struct Mask64 {
    data:u64,
}

impl Mask64 {
    pub fn new() -> Self {
        Mask64 {
            data: 0
        }
    }

    pub fn trailing_zeros(&self) -> u8 {
        self.data.trailing_zeros() as _
    }

    pub fn leading_zeros(&self) -> u8 {
        self.data.leading_zeros() as _
    }

    pub fn unclamped_trailing_zeros(&self, bit_index: u8) -> u8 {
        assert!(bit_index < 64);
        (self.data << (64 - bit_index)).leading_zeros() as _
    }

    pub fn unclamped_leading_zeros(&self, bit_index: u8) -> u8 {
        assert!(bit_index < 64);
        (self.data >> (bit_index + 1)).trailing_zeros() as _
    }

    pub fn set(&mut self, bit_index: u8) {
        assert!(bit_index < 64);
        self.data |= 1u64 << bit_index;
    }

    pub fn reset(&mut self, bit_index: u8) {
        assert!(bit_index < 64);
        self.data &= !(1u64 << bit_index);
    }
}