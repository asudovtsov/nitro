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

pub(crate) struct TypeErasedDeque<A: Allocator = Global> {
    blocks: Vec<*mut u8>,
    layout: Layout,
    len: usize,
    removed: HashSet<usize>,
    drop: unsafe fn(*mut u8),
    alloc: A,
}

impl TypeErasedDeque<Global> {
    pub fn new<T>() -> Self {
        Self::new_in::<T>(Global)
    }
}

impl<A: Allocator> TypeErasedDeque<A> {
    pub fn new_in<T>(alloc: A) -> Self {
        let layout = Layout::new::<T>();
        Self {
            blocks: vec![],
            layout,
            len: 0,
            removed: HashSet::new(),
            drop: |pointer: *mut u8| unsafe {
                drop(pointer.cast::<T>().read());
            },
            alloc,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn try_push<T>(&mut self, capacity: BlockCapacity, data: T) -> Option<usize> {
        if self.layout != Layout::new::<T>() {
            return None;
        }
        Some(self.push_unchecked(capacity, data))
    }

    pub unsafe fn push_unchecked<T>(&mut self, capacity: BlockCapacity, data: T) -> usize {
        debug_assert!(self.layout == Layout::new::<T>());
        let block_index = self.len / capacity.0;
        let inblock_index = self.len % capacity.0;
        if block_index >= self.blocks.len() {
            let block_layout = core::alloc::Layout::from_size_align_unchecked(
                self.layout.size() * capacity.0,
                self.layout.align(),
            );

            let pointer = self.alloc.allocate(block_layout).unwrap();
            self.blocks.push(&mut (*pointer.as_ptr())[0]);
        }
        let block = self.blocks.last().unwrap();
        let index = self.len;
        self.len += 1;
        block.cast::<T>().add(inblock_index).write(data);
        index
    }

    pub unsafe fn try_pop<T>(&mut self, capacity: BlockCapacity) -> Option<T> {
        if self.len == 0 || self.layout != Layout::new::<T>() {
            return None;
        }
        Some(self.pop_unchecked(capacity))
    }

    pub unsafe fn pop_unchecked<T>(&mut self, capacity: BlockCapacity) -> T {
        debug_assert!(self.layout == Layout::new::<T>());
        debug_assert!(self.len != 0);
        let block_index = self.len / capacity.0;
        let inblock_index = (self.len - 1) % capacity.0;
        let last_block = self.blocks[block_index];
        self.len -= 1;
        last_block.cast::<T>().add(inblock_index).read()
    }

    pub unsafe fn try_remove<T>(&mut self, capacity: BlockCapacity, index: usize) -> Option<T> {
        if index == self.len - 1 {
            return self.try_pop(capacity);
        }

        if index >= self.len || self.layout != Layout::new::<T>() || self.removed.contains(&index) {
            return None;
        }
        Some(self.remove_unchecked(capacity, index))
    }

    pub unsafe fn remove_unchecked<T>(&mut self, capacity: BlockCapacity, index: usize) -> T {
        debug_assert!(self.layout == Layout::new::<T>());
        debug_assert!(index < self.len);
        debug_assert!(!self.removed.contains(&index));
        let block_index = index / capacity.0;
        let inblock_index = index % capacity.0;
        let block = self.blocks[block_index];
        self.removed.insert(index);
        block.cast::<T>().add(inblock_index).read()
    }

    pub unsafe fn try_get<T>(&self, capacity: BlockCapacity, index: usize) -> Option<&T> {
        if index >= self.len || self.layout != Layout::new::<T>() || self.removed.contains(&index) {
            return None;
        }
        Some(self.get_ucnhecked(capacity, index))
    }

    pub unsafe fn get_ucnhecked<T>(&self, capacity: BlockCapacity, index: usize) -> &T {
        debug_assert!(self.layout == Layout::new::<T>());
        debug_assert!(index < self.len);
        debug_assert!(!self.removed.contains(&index));

        let block_index = index / capacity.0;
        let inblock_index = index % capacity.0;
        let block = self.blocks[block_index];
        &*block.cast::<T>().add(inblock_index)
    }

    pub unsafe fn try_get_mut<T>(
        &mut self,
        capacity: BlockCapacity,
        index: usize,
    ) -> Option<&mut T> {
        if index >= self.len || self.layout != Layout::new::<T>() || self.removed.contains(&index) {
            return None;
        }
        Some(self.get_mut_unchecked(capacity, index))
    }

    pub unsafe fn get_mut_unchecked<T>(&mut self, capacity: BlockCapacity, index: usize) -> &mut T {
        debug_assert!(self.layout == Layout::new::<T>());
        debug_assert!(index < self.len);
        debug_assert!(!self.removed.contains(&index));

        let block_index = index / capacity.0;
        let inblock_index = index % capacity.0;
        let block = self.blocks[block_index];
        &mut *block.cast::<T>().add(inblock_index)
    }

    pub unsafe fn shrink_to_fit(&mut self, capacity: BlockCapacity) {
        let free_block_count = (self.blocks.len() * capacity.0 - self.len) / capacity.0;
        let block_layout = core::alloc::Layout::from_size_align_unchecked(
            self.layout.size() * capacity.0,
            self.layout.align(),
        );

        for block in self.blocks.iter_mut().rev().take(free_block_count) {
            self.alloc
                .deallocate(NonNull::new_unchecked(*block), block_layout)
        }

        self.blocks.shrink_to_fit();
        self.removed.shrink_to_fit();
    }

    pub unsafe fn drop(deque: &mut Self, capacity: BlockCapacity) {
        if deque.len == 0 {
            return;
        }

        let mut index = deque.len;
        loop {
            if index == 0 {
                break;
            }

            index -= 1;

            if deque.removed.contains(&index) {
                continue;
            }

            let pointer = unsafe { deque.get_pointer_ucnhecked(capacity, index) };
            (deque.drop)(pointer);
        }

        let block_layout = core::alloc::Layout::from_size_align_unchecked(
            deque.layout.size() * capacity.0,
            deque.layout.align(),
        );

        for pointer in deque.blocks.iter_mut() {
            deque
                .alloc
                .deallocate(NonNull::new_unchecked(*pointer), block_layout)
        }
    }

    unsafe fn get_pointer_ucnhecked(&self, capacity: BlockCapacity, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        debug_assert!(!self.removed.contains(&index));

        let block_index = index / capacity.0;
        let inblock_index = index % capacity.0;
        let block = self.blocks[block_index];
        block.add(inblock_index)
    }
}
