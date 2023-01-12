use std::alloc::Layout;
use std::alloc;
use std::ptr::{NonNull, null_mut};
use std::mem;

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

pub(crate) struct RawArray<T> {
    data: *mut T
}

impl<T> RawArray<T> {
    pub fn uninit(size: usize) -> Self {
        let Ok(layout) = Layout::array::<T>(size) else {
            todo!()
        };

        unsafe {
            let data: *mut T = alloc::alloc(layout).cast();
            RawArray { data }
        }
    }

    pub unsafe fn drop_array(array: &mut RawArray<T>, size: usize) {
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

impl<T: Default> RawArray<T> {
    pub fn default_filled(size: usize) -> Self {
        let Ok(layout) = Layout::array::<T>(size) else {
            todo!()
        };

        unsafe {
            let data: *mut T = alloc::alloc(layout).cast();
            for i in 0..size {
                data.add(i).write(T::default());
            }

            RawArray { data }
        }
    }
}

// pub(crate) struct RawTable<T> {
//     data: RawArray<RawArray<T>>
// }

// impl<T> RawTable<T> {
//     pub unsafe fn drop_table(array: &mut RawTable<T>, row_count: usize, column_count: usize) {
//         for row in 0..row_count {
//             RawArray::<T>::drop_array(array.data.index_mut(row), column_count);
//         }
//         RawArray::<RawArray::<T>>::drop_array(&mut array.data, row_count)
//     }

//     pub unsafe fn index(&self, row: usize, column: usize) -> &T {
//         self.data.index(row).index(column)
//     }

//     pub unsafe fn index_mut(&mut self, row: usize, column: usize) -> &mut T {
//         self.data.index_mut(row).index_mut(column)
//     }

//     pub unsafe fn set(&mut self, row: usize, column: usize, value: T) {
//         self.data.index_mut(row).set(column, value);
//     }

//     pub unsafe fn replace(&mut self, row: usize, column: usize, value: T) -> T {
//         self.data.index_mut(row).replace(column, value)
//     }
// }

// impl<T: Clone> RawTable<T> {
//     pub fn from(row_count: usize, column_count: usize, value: T) -> Self {
//         let mut rows = RawArray::uninit(row_count);
//         for row in 0..row_count {
//             unsafe { rows.set(row, RawArray::new(column_count, value.clone())); }
//         }

//         RawTable { data: rows }
//     }
// }

type OptNode<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    prev: OptNode<T>,
    next: OptNode<T>,
    value: T,
}

impl<T> Node<T> {
    fn new(prev: OptNode<T>, next: OptNode<T>, value: T) -> Self {
        Node {
            prev,
            next,
            value
        }
    }

    fn unwrap(self) -> T {
        self.value
    }

    fn into_value(self: Box<Self>) -> T {
        self.value
    }

    fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

#[derive(Clone)]
pub(crate) struct LinkedList<T> {
    head: OptNode<T>
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: None
        }
    }

    pub fn push_front(&mut self, value: T) {
        let next = mem::take(&mut self.head);
        let ptr = Box::leak(Box::new(Node::new(None, next, value)));
        self.head = Some(unsafe {NonNull::new_unchecked(ptr)});
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let option = mem::take(&mut self.head);
        option?;

        let mut node = unsafe{Box::from_raw(option.unwrap().as_ptr())};
        self.head = mem::take(&mut node.next);
        Some(node.into_value())
    }

    pub fn cursor_front_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut { current: self.head, list: self }
    }
}

pub(crate) struct CursorMut<'a, T> {
    current: OptNode<T>,
    list: &'a mut LinkedList<T>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn current(&mut self) -> Option<&mut T> {
        if let Some(non_null) = self.current.as_mut() {
            return Some(unsafe{non_null.as_mut().as_mut()})
        }
        None
    }

    pub fn move_next(&mut self) {
        match self.current.take() {
            Some(current) => unsafe {
                self.current = current.as_ref().next;
            },
            None => {
                self.current = self.list.head;
            }
        }
    }

    pub fn remove_current(&mut self) -> Option<T> {
        let current_mut = unsafe { self.current?.as_mut() };
        let prev = current_mut.prev;
        let next = current_mut.next;
        match prev {
            Some(mut non_null) => unsafe { non_null.as_mut().next = next; },
            None => { self.list.head = next; }
        }
        unsafe { Some(Box::from_raw(current_mut).into_value()) }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{LinkedList, CursorMut};

    #[test]
    fn it_works() {
        let mut list = LinkedList::new();
        list.push_front("!");
        list.push_front("World");
        list.push_front("Hello ");

        let mut cursor = list.cursor_front_mut();
        cursor.move_next();
    }
}