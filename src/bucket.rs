use crate::params::{Size, UniqueTag};
use core::{alloc::Layout, marker::PhantomData, mem::ManuallyDrop, ptr::copy_nonoverlapping};
use std::alloc::{alloc, dealloc};

union CellData<T, S: Size> {
    data: ManuallyDrop<T>,
    index: ManuallyDrop<S>,
}

struct Cell<U: UniqueTag, T, S: Size> {
    tag: U,
    data: CellData<T, S>,
}

impl<U: UniqueTag, T, S: Size> Cell<U, T, S> {
    fn new(tag: U, data: T) -> Self {
        Self {
            tag,
            data: CellData {
                data: ManuallyDrop::new(data),
            },
        }
    }

    fn new_with_index(tag: U, index: S) -> Self {
        Self {
            tag,
            data: CellData {
                index: ManuallyDrop::new(index),
            },
        }
    }
}

pub(crate) struct Bucket<U: UniqueTag, S: Size> {
    cells: *mut u8,
    len: usize,
    capacity: usize,
    free_cursor: Option<S>,
    layout: Layout,
    drop_cell_fn: unsafe fn(*mut u8),
    reset_tag_fn: unsafe fn(*mut u8),
    contains_data_fn: unsafe fn(*mut u8) -> bool,
    compare_tag_fn: unsafe fn(*mut u8, tag: U) -> bool,
    phantom: PhantomData<(U, S)>,
}

impl<U: UniqueTag, S: Size> Bucket<U, S> {
    pub fn new<T>() -> Self {
        Self::with_capacity::<T>(0)
    }

    pub fn with_capacity<T>(capacity: usize) -> Self {
        let cells = if capacity != 0 {
            let array_layout = Layout::array::<Cell<U, T, S>>(capacity).unwrap();
            unsafe { std::alloc::alloc(array_layout) }
        } else {
            std::ptr::null_mut()
        };

        Self {
            cells,
            len: 0,
            capacity,
            free_cursor: None,
            layout: Layout::new::<Cell<U, T, S>>(),
            drop_cell_fn: |pointer| unsafe {
                let cell = pointer.cast::<Cell<U, T, S>>().read();
                debug_assert!(!cell.tag.is_removed());
                debug_assert!(!cell.tag.is_locked());
                ManuallyDrop::into_inner(cell.data.data);
            },
            reset_tag_fn: |pointer| unsafe {
                let cell = pointer.cast::<Cell<U, T, S>>().as_mut().unwrap();
                cell.tag = U::default();
            },
            contains_data_fn: |pointer| {
                let cell = unsafe { &*pointer.cast::<Cell<U, T, S>>() };
                !cell.tag.is_removed() && !cell.tag.is_locked()
            },
            compare_tag_fn: |pointer, tag| tag == unsafe { &*pointer.cast::<Cell<U, T, S>>() }.tag,
            phantom: Default::default(),
        }
    }

    pub fn try_place<T>(&mut self, data: T) -> Result<(S, U), T> {
        if self.layout != Layout::new::<T>() {
            return Err(data);
        }

        unsafe { self.place_unchecked(data) }
    }

    pub unsafe fn place_unchecked<T>(&mut self, data: T) -> Result<(S, U), T> {
        debug_assert!(self.layout == Layout::new::<Cell<U, T, S>>());

        // place to free cell
        if let Some(free_index) = self.free_cursor {
            let pointer = unsafe {
                self.cells
                    .cast::<Cell<U, T, S>>()
                    .add(free_index.into_usize())
            };

            let cell = unsafe { pointer.as_ref().unwrap() };
            debug_assert!(cell.tag.is_removed());

            self.free_cursor = if ManuallyDrop::into_inner(cell.data.index) != free_index {
                Some(ManuallyDrop::into_inner(cell.data.index))
            } else {
                None
            };

            let mut tag: U = cell.tag;
            tag.set_removed(false);
            unsafe { pointer.write(Cell::new(tag, data)) }
            return Ok((free_index, tag));
        }

        // place to new cell
        let index = self.len;

        // try grow if necessary
        if index == self.capacity && (self.capacity == S::max() || !self.try_grow::<T>()) {
            return Err(data);
        }

        let pointer = unsafe { self.cells.cast::<Cell<U, T, S>>().add(index) };
        unsafe { pointer.write(Cell::new(U::default(), data)) };
        self.len += 1;
        Ok((S::from_usize(index), U::default()))
    }

    pub fn try_remove<T>(&mut self, index: S) -> Option<T> {
        if self.layout != Layout::new::<T>() {
            return None;
        }

        unsafe { self.remove_unchecked(index) }
    }

    pub unsafe fn remove_unchecked<T>(&mut self, index: S) -> Option<T> {
        debug_assert!(self.layout == Layout::new::<Cell<U, T, S>>());

        if index.into_usize() >= self.len {
            return None;
        }

        let pointer = unsafe { self.cells.cast::<Cell<U, T, S>>().add(index.into_usize()) };
        let cell = unsafe { pointer.as_mut().unwrap() };
        if cell.tag.is_removed() || cell.tag.is_locked() {
            return None;
        }

        let data = {
            let cell = unsafe { pointer.read() };
            Some(ManuallyDrop::into_inner(cell.data.data))
        };

        let mut tag = cell.tag.next();
        if tag == cell.tag {
            tag.mark_locked();
            unsafe { pointer.write(Cell::new_with_index(tag, index)) }
            return data;
        }

        let prev_free_index = match self.free_cursor {
            Some(free_index) => free_index,
            None => index,
        };

        tag.set_removed(true);
        unsafe { pointer.write(Cell::new_with_index(tag, prev_free_index)) }
        self.free_cursor = Some(index);
        data
    }

    pub fn try_get<T>(&self, index: S) -> Option<&T> {
        if self.layout != Layout::new::<T>() {
            return None;
        }

        unsafe { self.get_unchecked(index) }
    }

    pub unsafe fn get_unchecked<T>(&self, index: S) -> Option<&T> {
        debug_assert!(self.layout == Layout::new::<Cell<U, T, S>>());

        if index.into_usize() >= self.len {
            return None;
        }

        let cell = unsafe { &*self.cells.cast::<Cell<U, T, S>>().add(index.into_usize()) };
        if cell.tag.is_removed() || cell.tag.is_locked() {
            return None;
        }

        Some(&cell.data.data)
    }

    pub fn try_get_mut<T>(&mut self, index: S) -> Option<&mut T> {
        if self.layout != Layout::new::<T>() {
            return None;
        }

        unsafe { self.get_mut_unchecked(index) }
    }

    pub unsafe fn get_mut_unchecked<T>(&mut self, index: S) -> Option<&mut T> {
        debug_assert!(self.layout == Layout::new::<Cell<U, T, S>>());

        if index.into_usize() >= self.len {
            return None;
        }

        let cell = unsafe { &mut *self.cells.cast::<Cell<U, T, S>>().add(index.into_usize()) };
        if cell.tag.is_removed() || cell.tag.is_locked() {
            return None;
        }

        Some(&mut cell.data.data)
    }

    pub unsafe fn contains(&self, tag: U, index: S) -> bool {
        if index.into_usize() >= self.len {
            return false;
        }
        unsafe {
            let pointer = self.get_pointer_unchecked(index.into_usize());
            (self.compare_tag_fn)(pointer, tag)
        }
    }

    fn try_grow<T>(&mut self) -> bool {
        if self.capacity == S::max() {
            return false;
        }

        let new_capacity = if self.capacity != 0 {
            usize::min(self.capacity << 1, S::max())
        } else {
            4 //#TODO setup start capacity
        };
        let layout = Layout::array::<Cell<U, T, S>>(new_capacity).unwrap();
        let pointer = unsafe { alloc(layout) };

        unsafe {
            copy_nonoverlapping(
                self.cells.cast::<Cell<U, T, S>>(),
                pointer.cast::<Cell<U, T, S>>(),
                self.len,
            );
        }

        if !self.cells.is_null() {
            unsafe { dealloc(self.cells, layout) }
        }
        self.cells = pointer;
        self.capacity = new_capacity;
        true
    }

    pub unsafe fn shrink_to_fit(&mut self) {
        todo!()
    }

    pub unsafe fn reset(&mut self) {
        if self.len == 0 {
            return;
        }

        let mut index = self.len;
        loop {
            if index == 0 {
                break;
            }

            index -= 1;

            unsafe {
                let pointer = self.get_pointer_unchecked(index);
                if (self.contains_data_fn)(pointer) {
                    (self.drop_cell_fn)(pointer);
                }
                (self.reset_tag_fn)(pointer)
            }
        }
    }

    pub unsafe fn clear(&mut self) -> bool {
        if self.len == 0 {
            return false;
        }

        let mut index = self.len;
        loop {
            if index == 0 {
                break;
            }

            index -= 1;

            let pointer = unsafe { self.get_pointer_unchecked(index) };
            if unsafe { (self.contains_data_fn)(pointer) } {
                unsafe { (self.drop_cell_fn)(pointer) };
            }
        }
        self.len = 0;
        true
    }

    pub unsafe fn drop(bucket: &mut Self) {
        if !Self::clear(bucket) {
            return;
        }

        let array_layout = core::alloc::Layout::from_size_align_unchecked(
            bucket.layout.size() * bucket.capacity,
            bucket.layout.align(),
        );

        unsafe { dealloc(bucket.cells, array_layout) }
    }

    unsafe fn get_pointer_unchecked(&self, index: usize) -> *mut u8 {
        let aligned = self.layout.pad_to_align();
        unsafe { self.cells.add(aligned.size() * index) }
    }
}
