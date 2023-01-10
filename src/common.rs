use std::fmt;
use std::mem;

#[derive(Debug, Copy, Clone)]
pub(crate) struct Valid<T: Sized> {
    ptr: *const T
}

impl<T: Sized> Valid<T> {
    pub fn new(ptr: *mut T) -> Option<Valid<T>> {
        if !ptr.is_null() && mem::size_of::<T>() != 0 /* && ptr.is_aligned() #TODO */ {
            Some(Valid{ptr})
        } else {
            None
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr.cast_mut() }
    }
}

impl<T> Eq for Valid<T> {}

impl<T> PartialEq for Valid<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> fmt::Pointer for Valid<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr, f)
    }
}