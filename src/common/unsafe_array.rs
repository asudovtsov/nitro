use std::alloc::Layout;
use std::alloc;
use std::mem;

pub(crate) struct UnsafeArray<T> {
    data: *mut T
}

impl<T> UnsafeArray<T> {
    pub fn new<F>(size: usize, f: F) -> Self
        where F: Fn() -> T
    {
        let Ok(layout) = Layout::array::<T>(size) else {
            todo!()
        };

        unsafe {
            let data: *mut T = alloc::alloc(layout).cast();
            for i in 0..size {
                data.add(i).write(f());
            }

            UnsafeArray { data }
        }
    }

    pub fn uninit(size: usize) -> Self {
        let Ok(layout) = Layout::array::<T>(size) else {
            todo!()
        };

        unsafe {
            let data: *mut T = alloc::alloc(layout).cast();
            UnsafeArray { data }
        }
    }

    pub unsafe fn drop_array(array: &mut UnsafeArray<T>, size: usize) {
        let Ok(layout) = Layout::array::<T>(size) else {
            todo!()
        };

        array.data.drop_in_place();
        alloc::dealloc(array.data.cast(), layout);
    }

    pub unsafe fn index(&self, index: usize) -> &T {
        &*self.data.add(index).cast()
    }

    pub unsafe fn index_mut(&mut self, index: usize) -> &mut T {
        &mut *self.data.add(index).cast()
    }

    pub unsafe fn set(&mut self, index: usize, value: T) {
        self.data.add(index).write(value);
    }

    pub unsafe fn replace(&mut self, index: usize, value: T) -> T {
        mem::replace(&mut *self.data.add(index).cast(), value)
    }
}