use std::alloc::Layout;
use std::alloc;
use std::ptr::NonNull;
use std::fmt;

// #[derive(Debug, Copy, Clone)]
// pub(crate) struct Strict<T> {
//     non_null: NonNull<T>
// }

// impl<T> Strict<T> {
//     pub fn new(ptr: *mut T) -> Option<Strict<T>> {
//         if !ptr.is_null() /* && ptr.is_aligned() #TODO */ {
//             Some(Strict{non_null: unsafe { NonNull::new_unchecked(ptr) }})
//         } else {
//             None
//         }
//     }

//     pub fn as_ref(&self) -> &T {
//         unsafe { self.non_null.as_ref() }
//     }

//     pub fn as_mut(&mut self) -> &mut T {
//         unsafe { self.non_null.as_mut() }
//     }

//     pub fn as_ptr(&mut self) -> *mut T {
//         self.non_null.as_ptr()
//     }
// }

// impl<T> Eq for Strict<T> {}

// impl<T> PartialEq for Strict<T> {
//     #[inline]
//     fn eq(&self, other: &Self) -> bool {
//         self.non_null == other.non_null
//     }
// }

// impl<T> fmt::Pointer for Strict<T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         fmt::Pointer::fmt(&self.non_null, f)
//     }
// }

pub(crate) struct UnsafeArray<T> {
    data: *mut T
}

impl<T> UnsafeArray<T> {
    pub fn uninit(size: usize) -> Self {
        let Ok(layout) = Layout::array::<T>(size) else {
            todo!()
        };

        //#TODO is it necessary?
        // let Ok(layout) = layout.align_to(mem::align_of::<T>()) else {
        //     panic!("align error")
        // };

        unsafe {
            let data: *mut T = alloc::alloc(layout).cast();
            UnsafeArray { data }
        }
    }

    pub unsafe fn drop_array(array: &mut UnsafeArray<T>, size: usize) {
        let Ok(layout) = Layout::array::<T>(size) else {
            todo!()
        };

        //#TODO is it necessary?
        // let Ok(layout) = layout.align_to(mem::align_of::<T>()) else {
        //     panic!("align error")
        // };

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
        std::mem::replace(&mut *self.data.add(index).cast(), value)
    }
}

impl<T: Clone> UnsafeArray<T> {
    pub fn from(size: usize, value: T) -> Self {
        let Ok(layout) = Layout::array::<T>(size) else {
            todo!()
        };

        //#TODO is it necessary?
        // let Ok(layout) = layout.align_to(mem::align_of::<T>()) else {
        //     panic!("align error")
        // };

        unsafe {
            let data: *mut T = alloc::alloc(layout).cast();
            for i in 0..size {
                data.add(i).write(value.clone());
            }

            UnsafeArray { data }
        }
    }
}

pub(crate) struct UnsafeTable<T> {
    data: UnsafeArray<UnsafeArray<T>>
}

impl<T> UnsafeTable<T> {
    pub unsafe fn drop_table(array: &mut UnsafeTable<T>, row_count: usize, column_count: usize) {
        for row in 0..row_count {
            UnsafeArray::<T>::drop_array(array.data.index_mut(row), column_count);
        }
        UnsafeArray::<UnsafeArray::<T>>::drop_array(&mut array.data, row_count)
    }

    pub unsafe fn index(&self, row: usize, column: usize) -> &T {
        self.data.index(row).index(column)
    }

    pub unsafe fn index_mut(&mut self, row: usize, column: usize) -> &mut T {
        self.data.index_mut(row).index_mut(column)
    }

    pub unsafe fn set(&mut self, row: usize, column: usize, value: T) {
        self.data.index_mut(row).set(column, value);
    }

    pub unsafe fn replace(&mut self, row: usize, column: usize, value: T) -> T {
        self.data.index_mut(row).replace(column, value)
    }
}

impl<T: Clone> UnsafeTable<T> {
    pub fn from(row_count: usize, column_count: usize, value: T) -> Self {
        let mut rows = UnsafeArray::uninit(row_count);
        for row in 0..row_count {
            unsafe { rows.set(row, UnsafeArray::from(column_count, value.clone())); }
        }

        UnsafeTable { data: rows }
    }
}

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    value: T,
}

impl<T> Node<T> {
    fn unwrap(self) -> T {
        self.value
    }

    fn into_value(self: Box<Self>) -> T {
        self.value
    }

    fn take_next(&mut self) -> T {

    }
}

pub(crate) struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>
}

impl<T: Default> LinkedList<T> {
    fn new() -> Self {
        LinkedList {
            head: None
        }
    }

    fn push_front(&mut self, value: T) {
        let next = std::mem::take(&mut self.head);
        let ptr = Box::leak(Box::new(Node { next, value }));
        self.head = Some(unsafe {NonNull::new_unchecked(ptr)});
    }

    fn pop_front(&mut self) -> Option<T> {
        let option = std::mem::take(&mut self.head);
        option?;

        let mut node = unsafe{Box::from_raw(option.unwrap().as_ptr())};
        self.head = std::mem::take(&mut node.next);
        Some(node.into_value())
    }
}