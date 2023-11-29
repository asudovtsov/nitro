use crate::tag::UniqueTag;
use allocator_api2::alloc::{Allocator, Global};
use core::alloc::Layout;
use core::ptr::NonNull;
use std::collections::HashSet;
use std::marker::PhantomData;

struct Cell<U: UniqueTag, T> {
    tag: U,
    data: T,
}

impl<U: UniqueTag, T> Cell<U, T> {
    fn new(tag: U, data: T) -> Self {
        Self { tag, data }
    }
}

pub(crate) struct Bucket<U: UniqueTag, A: Allocator = Global> {
    blocks: Vec<*mut u8>,
    layout: Layout,
    len: usize,
    cell_count: usize,
    removed: HashSet<usize>,
    is_banned_cell: unsafe fn(*mut u8) -> bool,
    reset_cell_tag: unsafe fn(*mut u8),
    drop_cell: unsafe fn(*mut u8),
    alloc: A,
    phantom: PhantomData<U>,
}

impl<U: UniqueTag> Bucket<U, Global> {
    pub fn new<T>() -> Self {
        Self::new_in::<T>(Global)
    }
}

impl<U: UniqueTag, A: Allocator> Bucket<U, A> {
    pub fn new_in<T>(alloc: A) -> Self {
        let layout = Layout::new::<Cell<U, T>>();
        Self {
            blocks: vec![],
            layout,
            len: 0,
            cell_count: 0,
            removed: HashSet::new(),
            is_banned_cell: |pointer: *mut u8| unsafe {
                (*pointer.cast::<Cell<U, T>>()).tag.is_over()
            },
            reset_cell_tag: |pointer: *mut u8| unsafe {
                (*pointer.cast::<Cell<U, T>>()).tag = Default::default();
            },
            drop_cell: |pointer: *mut u8| unsafe {
                pointer.cast::<Cell<U, T>>().read();
            },
            alloc,
            phantom: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn try_place<T>(&mut self, capacity: usize, data: T) -> Option<(usize, U)> {
        if self.layout != Layout::new::<Cell<U, T>>() {
            return None;
        }
        unsafe { Some(self.place_unchecked(capacity, data)) }
    }

    pub unsafe fn place_unchecked<T>(&mut self, capacity: usize, data: T) -> (usize, U) {
        debug_assert!(self.layout == Layout::new::<Cell<U, T>>());

        let mut index = self.next_index();
        loop {
            let block_index = index / capacity;
            if block_index >= self.blocks.len() {
                unsafe {
                    self.grow(capacity);
                }
            }

            let (pointer, exists) = unsafe { self.get_pointer_ucnhecked_for::<T>(capacity, index) };
            if exists && unsafe { (self.is_banned_cell)(pointer.cast()) } {
                if index == self.len {
                    self.len += 1;
                }
                index += 1;
                continue;
            }

            let tag = if exists {
                unsafe { (*pointer).tag }
            } else {
                Default::default()
            };

            if tag.is_over() {
                if index == self.len {
                    self.len += 1;
                }
                index += 1;
                continue;
            }

            unsafe {
                pointer.write(Cell::new(tag.next(), data));
            }
            self.removed.remove(&index);
            if index == self.len {
                self.len += 1;
            }

            if self.len > self.cell_count {
                self.cell_count = self.len;
            }

            return (index, tag);
        }
    }

    pub unsafe fn try_remove<T>(&mut self, capacity: usize, index: usize) -> Option<T> {
        if self.layout != Layout::new::<Cell<U, T>>() {
            return None;
        }
        unsafe { self.remove_unchecked(capacity, index) }
    }

    pub unsafe fn remove_unchecked<T>(&mut self, capacity: usize, index: usize) -> Option<T> {
        debug_assert!(self.layout == Layout::new::<Cell<U, T>>());
        if index >= self.len || self.removed.contains(&index) {
            return None;
        }

        if index == self.len - 1 {
            self.len -= 1;
        }

        let (pointer, exists) = unsafe { self.get_pointer_ucnhecked_for::<T>(capacity, index) };
        debug_assert!(exists);

        if (self.is_banned_cell)(pointer.cast()) {
            return None;
        }

        self.removed.insert(index);
        unsafe { Some(pointer.read().data) }
    }

    pub unsafe fn try_get<T>(&self, capacity: usize, index: usize) -> Option<&T> {
        if self.layout != Layout::new::<Cell<U, T>>() {
            return None;
        }
        unsafe { self.get_ucnhecked(capacity, index) }
    }

    pub unsafe fn get_ucnhecked<T>(&self, capacity: usize, index: usize) -> Option<&T> {
        debug_assert!(self.layout == Layout::new::<Cell<U, T>>());
        if index >= self.len || self.removed.contains(&index) {
            return None;
        }

        let (pointer, exists) = unsafe { self.get_pointer_ucnhecked_for::<T>(capacity, index) };
        debug_assert!(exists);

        if unsafe { (self.is_banned_cell)(pointer.cast()) } {
            return None;
        }

        unsafe { Some(&(*pointer).data) }
    }

    pub unsafe fn try_get_mut<T>(&mut self, capacity: usize, index: usize) -> Option<&mut T> {
        if self.layout != Layout::new::<Cell<U, T>>() {
            return None;
        }
        unsafe { self.get_mut_unchecked(capacity, index) }
    }

    pub unsafe fn get_mut_unchecked<T>(&mut self, capacity: usize, index: usize) -> Option<&mut T> {
        debug_assert!(self.layout == Layout::new::<Cell<U, T>>());
        if index >= self.len || self.removed.contains(&index) {
            return None;
        }

        let (pointer, exists) = unsafe { self.get_pointer_ucnhecked_for::<T>(capacity, index) };
        debug_assert!(exists);

        if unsafe { (self.is_banned_cell)(pointer.cast()) } {
            return None;
        }

        unsafe { Some(&mut (*pointer).data) }
    }

    pub fn contains(&self, capacity: usize, index: usize) -> bool {
        if index >= self.len {
            return false;
        }
        unsafe {
            let (pointer, exists) = self.get_pointer_ucnhecked(capacity, index);
            debug_assert!(exists);

            !(self.is_banned_cell)(pointer) && !self.removed.contains(&index)
        }
    }

    pub unsafe fn shrink_to_fit(&mut self, capacity: usize) {
        let free_block_count = (self.cell_count - self.len) / capacity;
        let block_layout = unsafe {
            core::alloc::Layout::from_size_align_unchecked(
                self.layout.size() * capacity,
                self.layout.align(),
            )
        };

        for block in self.blocks.iter_mut().rev().take(free_block_count) {
            unsafe {
                self.alloc
                    .deallocate(NonNull::new_unchecked(*block), block_layout)
            }
        }

        self.cell_count = self.len;
        self.blocks.shrink_to_fit();
        self.removed.shrink_to_fit();
    }

    pub unsafe fn reset(bucket: &mut Self, capacity: usize) {
        if bucket.len == 0 {
            return;
        }

        let mut index = bucket.len;
        loop {
            if index == 0 {
                break;
            }

            index -= 1;

            let (pointer, exists) = unsafe { bucket.get_pointer_ucnhecked(capacity, index) };
            debug_assert!(exists);

            if !bucket.removed.contains(&index) && unsafe { !(bucket.is_banned_cell)(pointer) } {
                unsafe {
                    (bucket.drop_cell)(pointer);
                }
            }
            unsafe {
                (bucket.reset_cell_tag)(pointer);
            }
        }

        bucket.removed.clear();
    }

    pub unsafe fn clear(bucket: &mut Self, capacity: usize) -> bool {
        if bucket.len == 0 {
            return false;
        }

        let mut index = bucket.len;
        loop {
            if index == 0 {
                break;
            }

            index -= 1;

            let (pointer, exists) = unsafe { bucket.get_pointer_ucnhecked(capacity, index) };
            debug_assert!(exists);

            if unsafe { (bucket.is_banned_cell)(pointer) } || bucket.removed.contains(&index) {
                continue;
            }
            unsafe { (bucket.drop_cell)(pointer) };
        }
        bucket.len = 0;
        true
    }

    pub unsafe fn drop(bucket: &mut Self, capacity: usize) {
        if !Self::clear(bucket, capacity) {
            return;
        }

        let block_layout = core::alloc::Layout::from_size_align_unchecked(
            bucket.layout.size() * capacity,
            bucket.layout.align(),
        );

        for pointer in bucket.blocks.iter_mut() {
            bucket
                .alloc
                .deallocate(NonNull::new_unchecked(*pointer), block_layout)
        }
    }

    unsafe fn grow(&mut self, capacity: usize) {
        let block_layout = unsafe {
            core::alloc::Layout::from_size_align_unchecked(
                self.layout.size() * capacity,
                self.layout.align(),
            )
        };

        let pointer = self.alloc.allocate(block_layout).unwrap().cast::<u8>();
        self.blocks.push(pointer.as_ptr());
    }

    unsafe fn get_pointer_ucnhecked(&self, capacity: usize, index: usize) -> (*mut u8, bool) {
        let block_index = index / capacity;
        let inblock_index = index % capacity;
        let block = self.blocks[block_index];
        let aligned = self.layout.pad_to_align();
        (
            unsafe { block.add(aligned.size() * inblock_index) },
            index < self.cell_count,
        )
    }

    unsafe fn get_pointer_ucnhecked_for<T>(
        &self,
        capacity: usize,
        index: usize,
    ) -> (*mut Cell<U, T>, bool) {
        let block_index = index / capacity;
        let inblock_index = index % capacity;
        let block = self.blocks[block_index];
        (
            unsafe { block.cast::<Cell<U, T>>().add(inblock_index) },
            index < self.cell_count,
        )
    }

    fn next_index(&mut self) -> usize {
        if let Some(index) = self.removed.iter().next().cloned() {
            return index;
        }
        self.len
    }
}
