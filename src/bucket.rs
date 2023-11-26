use allocator_api2::alloc::{Allocator, Global};
use core::alloc::Layout;
use core::ptr::NonNull;
use std::collections::HashSet;

#[derive(Copy, Clone)]
pub(crate) struct BlockCapacity(usize);

impl BlockCapacity {
    pub fn new(capacity: usize) -> Self {
        Self(capacity)
    }
}

struct Cell<T> {
    gen: u64,
    data: T,
}

impl<T> Cell<T> {
    fn new(gen: u64, data: T) -> Self {
        Self { gen, data }
    }
}

pub(crate) struct Bucket<A: Allocator = Global> {
    blocks: Vec<*mut u8>,
    layout: Layout,
    len: usize,
    cell_len: usize,
    removed: HashSet<usize>,
    dead: HashSet<usize>,
    drop: unsafe fn(*mut u8),
    alloc: A,
}

impl Bucket<Global> {
    pub fn new<T>() -> Self {
        Self::new_in::<T>(Global)
    }
}

impl<A: Allocator> Bucket<A> {
    pub fn new_in<T>(alloc: A) -> Self {
        let layout = Layout::new::<Cell<T>>();
        Self {
            blocks: vec![],
            layout,
            len: 0,
            cell_len: 0,
            removed: HashSet::new(),
            dead: HashSet::new(),
            drop: |pointer: *mut u8| unsafe {
                pointer.cast::<Cell<T>>().read();
            },
            alloc,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn dead_count(&self) -> usize {
        self.dead.len()
    }

    pub unsafe fn try_place<T>(
        &mut self,
        capacity: BlockCapacity,
        data: T,
    ) -> Option<(usize, u64)> {
        if self.layout != Layout::new::<Cell<T>>() {
            return None;
        }
        Some(self.place_unchecked(capacity, data))
    }

    pub unsafe fn place_unchecked<T>(&mut self, capacity: BlockCapacity, data: T) -> (usize, u64) {
        debug_assert!(self.layout == Layout::new::<Cell<T>>());

        let mut index = self.next_index();
        loop {
            let block_index = index / capacity.0;
            if block_index >= self.blocks.len() {
                self.grow(capacity);
            }

            if self.dead.contains(&index) {
                if index == self.len {
                    self.len += 1;
                }
                index += 1;
                continue;
            }

            let inblock_index = index % capacity.0;
            let block = self.blocks.last().unwrap();
            let pointer = block.cast::<Cell<T>>().add(inblock_index);

            let gen = if index < self.cell_len {
                (*pointer).gen
            } else {
                0
            };

            if gen == u64::MAX {
                self.dead.insert(index);
                if index == self.len {
                    self.len += 1;
                }
                index += 1;
                continue;
            }

            pointer.write(Cell::new(gen + 1, data));
            self.removed.remove(&index);
            if index == self.len {
                self.len += 1;
            }

            if self.len > self.cell_len {
                self.cell_len = self.len;
            }

            return (index, gen);
        }
    }

    pub unsafe fn try_remove<T>(&mut self, capacity: BlockCapacity, index: usize) -> Option<T> {
        if self.layout != Layout::new::<Cell<T>>() {
            return None;
        }
        self.remove_unchecked(capacity, index)
    }

    pub unsafe fn remove_unchecked<T>(
        &mut self,
        capacity: BlockCapacity,
        index: usize,
    ) -> Option<T> {
        debug_assert!(self.layout == Layout::new::<Cell<T>>());
        if index >= self.len || self.removed.contains(&index) || self.dead.contains(&index) {
            return None;
        }

        if index == self.len - 1 {
            self.len -= 1;
        }

        let block_index = index / capacity.0;
        let inblock_index = index % capacity.0;
        let block = self.blocks[block_index];

        self.removed.insert(index);
        Some(block.cast::<Cell<T>>().add(inblock_index).read().data)
    }

    pub unsafe fn try_get<T>(&self, capacity: BlockCapacity, index: usize) -> Option<&T> {
        if self.layout != Layout::new::<Cell<T>>() {
            return None;
        }
        self.get_ucnhecked(capacity, index)
    }

    pub unsafe fn get_ucnhecked<T>(&self, capacity: BlockCapacity, index: usize) -> Option<&T> {
        debug_assert!(self.layout == Layout::new::<Cell<T>>());
        if index >= self.len || self.removed.contains(&index) || self.dead.contains(&index) {
            return None;
        }

        let block_index = index / capacity.0;
        let inblock_index = index % capacity.0;

        let block = self.blocks[block_index];
        Some(&(*block.cast::<Cell<T>>().add(inblock_index)).data)
    }

    pub unsafe fn try_get_mut<T>(
        &mut self,
        capacity: BlockCapacity,
        index: usize,
    ) -> Option<&mut T> {
        if self.layout != Layout::new::<Cell<T>>() {
            return None;
        }
        self.get_mut_unchecked(capacity, index)
    }

    pub unsafe fn get_mut_unchecked<T>(
        &mut self,
        capacity: BlockCapacity,
        index: usize,
    ) -> Option<&mut T> {
        debug_assert!(self.layout == Layout::new::<Cell<T>>());
        if index >= self.len || self.removed.contains(&index) || self.dead.contains(&index) {
            return None;
        }

        let block_index = index / capacity.0;
        let inblock_index = index % capacity.0;
        let block = self.blocks[block_index];
        Some(&mut (*block.cast::<Cell<T>>().add(inblock_index)).data)
    }

    pub fn contains(&self, index: usize) -> bool {
        index < self.len && !self.removed.contains(&index) && !self.dead.contains(&index)
    }

    pub unsafe fn shrink_to_fit(&mut self, capacity: BlockCapacity) {
        let free_block_count = (self.cell_len - self.len) / capacity.0;
        let block_layout = core::alloc::Layout::from_size_align_unchecked(
            self.layout.size() * capacity.0,
            self.layout.align(),
        );

        for block in self.blocks.iter_mut().rev().take(free_block_count) {
            self.alloc
                .deallocate(NonNull::new_unchecked(*block), block_layout)
        }

        self.cell_len = self.len;
        self.blocks.shrink_to_fit();
        self.removed.shrink_to_fit();
    }

    pub unsafe fn drop(bucket: &mut Self, capacity: BlockCapacity) {
        if bucket.len == 0 {
            return;
        }

        let mut index = bucket.len;
        loop {
            if index == 0 {
                break;
            }

            index -= 1;

            if bucket.removed.contains(&index) || bucket.dead.contains(&index) {
                continue;
            }

            let pointer = unsafe { bucket.get_pointer_ucnhecked(capacity, index) };
            (bucket.drop)(pointer);
        }

        let block_layout = core::alloc::Layout::from_size_align_unchecked(
            bucket.layout.size() * capacity.0,
            bucket.layout.align(),
        );

        for pointer in bucket.blocks.iter_mut() {
            bucket
                .alloc
                .deallocate(NonNull::new_unchecked(*pointer), block_layout)
        }
    }

    unsafe fn grow(&mut self, capacity: BlockCapacity) {
        let block_layout = core::alloc::Layout::from_size_align_unchecked(
            self.layout.size() * capacity.0,
            self.layout.align(),
        );

        let pointer = self.alloc.allocate(block_layout).unwrap().cast::<u8>();
        self.blocks.push(pointer.as_ptr());
    }

    unsafe fn get_pointer_ucnhecked(&self, capacity: BlockCapacity, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        debug_assert!(!self.removed.contains(&index));

        let block_index = index / capacity.0;
        let inblock_index = index % capacity.0;
        let block = self.blocks[block_index];
        let aligned = self.layout.pad_to_align();
        block.add(aligned.size() * inblock_index)
    }

    fn next_index(&mut self) -> usize {
        if let Some(index) = self.removed.iter().next().cloned() {
            return index;
        }
        self.len
    }
}
