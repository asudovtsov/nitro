use crate::params::Size;
use core::{alloc::Layout, marker::PhantomData, ptr::copy_nonoverlapping};
use std::alloc::{alloc, dealloc};

struct Cell<T, S: Size> {
    data: T,
    token_index: S,
}

impl<T, S: Size> Cell<T, S> {
    fn new(data: T, token_index: S) -> Self {
        Self { data, token_index }
    }
}

pub(crate) struct Bucket<S: Size> {
    data: *mut u8,
    layout: Layout,
    capacity: usize,
    len: usize,
    drop_fn: unsafe fn(*mut u8),
    swap_fn: unsafe fn(*mut u8, *mut u8),
    get_token_index_fn: unsafe fn(*mut u8) -> S,
    phantom: PhantomData<S>,
}

impl<S: Size> Bucket<S> {
    pub fn new<T>() -> Self {
        Self::with_capacity::<T>(0)
    }

    pub fn with_capacity<T>(capacity: usize) -> Self {
        let data = if capacity != 0 {
            let array_layout = Layout::array::<Cell<T, S>>(capacity).unwrap();
            unsafe { std::alloc::alloc(array_layout) }
        } else {
            std::ptr::null_mut()
        };

        Self {
            data,
            layout: Layout::new::<Cell<T, S>>(),
            capacity,
            len: 0,
            drop_fn: |pointer| unsafe {
                pointer.cast::<Cell<T, S>>().read();
            },
            swap_fn: |l, r| unsafe { l.cast::<Cell<T, S>>().swap(r.cast::<Cell<T, S>>()) },
            get_token_index_fn: |pointer| unsafe { (*pointer.cast::<Cell<T, S>>()).token_index },
            phantom: Default::default(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // set_token_index_unchecked must be called after push
    pub unsafe fn push_unchecked<T: 'static>(&mut self, data: T) -> Result<S, T> {
        debug_assert!(self.layout == Layout::new::<Cell<T, S>>());

        if self.len == self.capacity && !self.try_grow::<Cell<T, S>>() {
            return Err(data);
        }

        let index = self.len;
        let pointer = unsafe { self.data.cast::<Cell<T, S>>().add(index) };
        unsafe { pointer.write(Cell::new(data, 0.into())) };
        self.len += 1;
        Ok(index.into())
    }

    pub unsafe fn set_token_index_unchecked<T: 'static>(&mut self, index: S, token_index: S) {
        let usize_index = index.into();
        debug_assert!(usize_index < self.len);

        unsafe {
            let cell = &mut *self.data.cast::<Cell<T, S>>().add(usize_index);
            cell.token_index = token_index;
        }
    }

    pub unsafe fn swap_remove_unchecked<T: 'static>(&mut self, index: S) -> (T, Option<S>) {
        let usize_index = index.into();
        debug_assert!(usize_index < self.len);

        let pointer_to_last = unsafe { self.data.cast::<Cell<T, S>>().add(self.len - 1) };
        if usize_index == self.len - 1 {
            self.len -= 1;
            return unsafe {
                let cell = pointer_to_last.read();
                (cell.data, None)
            };
        }

        let pointer = unsafe { self.data.cast::<Cell<T, S>>().add(usize_index) };
        unsafe { pointer.swap(pointer_to_last) }

        self.len -= 1;
        unsafe {
            let cell = &*pointer;
            let removed_cell = pointer_to_last.read(); // drop data
            (removed_cell.data, Some(cell.token_index))
        }
    }

    pub unsafe fn swap_erase_unchecked(&mut self, index: S) -> Option<S> {
        let usize_index = index.into();
        debug_assert!(usize_index < self.len);

        let pointer_to_last = self.get_pointer_unchecked(self.len - 1);
        if usize_index == self.len - 1 {
            self.len -= 1;
            (self.drop_fn)(pointer_to_last);
            return None;
        }

        let pointer = self.get_pointer_unchecked(usize_index);
        (self.swap_fn)(pointer, pointer_to_last);
        (self.drop_fn)(pointer_to_last);
        self.len -= 1;
        unsafe { Some((self.get_token_index_fn)(pointer)) }
    }

    pub fn try_get<T>(&self, index: S) -> Option<&T> {
        if self.layout != Layout::new::<Cell<T, S>>() {
            return None;
        }

        if index.into() >= self.len {
            return None;
        }

        unsafe { Some(self.get_unchecked(index)) }
    }

    pub unsafe fn get_unchecked<T>(&self, index: S) -> &T {
        debug_assert!(self.layout == Layout::new::<Cell<T, S>>());
        debug_assert!(index.into() < self.len);

        unsafe {
            let cell = &*self.data.cast::<Cell<T, S>>().add(index.into());
            &cell.data
        }
    }

    pub fn try_get_mut<T>(&mut self, index: S) -> Option<&mut T> {
        if self.layout != Layout::new::<Cell<T, S>>() {
            return None;
        }

        if index.into() >= self.len {
            return None;
        }

        unsafe { Some(self.get_mut_unchecked(index)) }
    }

    pub unsafe fn get_mut_unchecked<T>(&mut self, index: S) -> &mut T {
        debug_assert!(self.layout == Layout::new::<Cell<T, S>>());
        debug_assert!(index.into() < self.len);

        unsafe {
            let cell = &mut *self.data.cast::<Cell<T, S>>().add(index.into());
            &mut cell.data
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

        let layout = Layout::array::<Cell<T, S>>(new_capacity).unwrap();
        let pointer = unsafe { alloc(layout) };

        unsafe {
            copy_nonoverlapping(
                self.data.cast::<Cell<T, S>>(),
                pointer.cast::<Cell<T, S>>(),
                self.len,
            );
        }

        if !self.data.is_null() {
            unsafe { dealloc(self.data, layout) }
        }

        self.data = pointer;
        self.capacity = new_capacity;
        true
    }

    pub unsafe fn shrink_to_fit(&mut self) {
        todo!()
        // if self.capacity == 0 {
        //     return;
        // }

        // let (layout, size) = self.layout.repeat(self.len).unwrap();
        // assert_eq!(size, self.len);

        // let mut pointer = std::ptr::null_mut();
        // if self.len != 0 {
        //     pointer = unsafe { alloc(layout) };
        //     unsafe { copy_nonoverlapping(self.data, pointer, layout.size() * self.len) }
        // }

        // if !self.data.is_null() {
        //     unsafe { dealloc(self.data, layout) }
        // }

        // self.data = pointer;
        // self.capacity = self.len;
    }

    pub unsafe fn clear(&mut self) {
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
                (self.drop_fn)(pointer)
            }
        }
    }

    pub unsafe fn drop(bucket: &mut Self) {
        Self::clear(bucket);

        if Self::capacity(bucket) == 0 {
            return;
        }

        let array_layout = core::alloc::Layout::from_size_align_unchecked(
            bucket.layout.size() * bucket.capacity,
            bucket.layout.align(),
        );

        unsafe { dealloc(bucket.data, array_layout) }
    }

    unsafe fn get_pointer_unchecked(&self, index: usize) -> *mut u8 {
        let aligned = self.layout.pad_to_align();
        unsafe { self.data.add(aligned.size() * index) }
    }
}
