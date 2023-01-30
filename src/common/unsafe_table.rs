use std::ptr::NonNull;

use crate::block;
use crate::common::linked_list::{LinkedList, Node, OptNode, CursorMut as ColumnCursorMut};
use crate::common::unsafe_array::UnsafeArray;

pub(crate) struct Deq<T> {
    data: Vec<UnsafeArray<T>>,
    len: usize,
    block_capacity: usize,
}

impl<T> Deq<T> {
    pub fn new<F>(block_capacity: usize, f: F) -> Self
        where F: Fn() -> T
    {
        assert_ne!(block_capacity, 0);
        Deq {
            data: vec![UnsafeArray::new(block_capacity, f)],
            len: 0,
            block_capacity
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.block_capacity * self.data.len()
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.capacity() {
            self.data.push(UnsafeArray::uninit(self.block_capacity));
        }

        let block_index = self.len / self.block_capacity;
        unsafe { self.data[block_index].set(self.len % self.block_capacity, value) };
    }

    pub unsafe fn index(&self, index: usize) -> &T {
        assert!(index < self.len);
        let block_index = index / self.block_capacity;
        unsafe { self.data[block_index].index(index % self.block_capacity) }
    }

    pub unsafe fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.len);
        let block_index = index / self.block_capacity;
        unsafe { self.data[block_index].index_mut(index % self.block_capacity) }
    }
}

impl<T> Drop for Deq<T> {
    fn drop(&mut self) {
        let block_index = self.len / self.block_capacity;
        for (i, array) in self.data.iter_mut().enumerate() {
            let len = if i < block_index {
                self.block_capacity
            } else {
                self.len % self.block_capacity
            };
            unsafe {
                UnsafeArray::drop_array(array, len, self.block_capacity);
            }
        }
    }
}

pub(crate) struct UnsafeTable<T> {
    array: UnsafeArray<LinkedList<T>>,
    deq: Deq<OptNode<T>>,
}

impl<T> UnsafeTable<T> {
    pub fn new(width: usize, capacity: usize) -> Self {
        UnsafeTable {
            array: UnsafeArray::new(width, LinkedList::new),
            deq: Deq::new(capacity, || None)
        }
    }

    // pub unsafe fn push_front(&mut self, column: usize, value: T) {
    //     self.array.index_mut(column).push_front(value);
    // }

    // pub unsafe fn pop_front(&mut self, column: usize) -> Option<T> {
    //     self.array.index_mut(column).pop_front()
    // }

    pub unsafe fn push_node_front(&mut self, column: usize, node: NonNull<Node<T>>) {
        self.array.index_mut(column).push_node_front(node);
    }

    pub unsafe fn pop_node_front(&mut self, column: usize) -> OptNode<T> {
        self.array.index_mut(column).pop_node_front()
    }

    pub unsafe fn column_cursor_front_mut(&mut self, column: usize) -> ColumnCursorMut<'_, T> {
        self.array.index_mut(column).cursor_front_mut()
    }

    pub unsafe fn cursor_mut(&mut self, column: usize, table_width: usize) -> CursorMut<'_, T> {
        assert_ne!(table_width, 0);
        CursorMut {
            current: unsafe{self.array.index_mut(column)}.head(),
            table: self,
            column,
            table_width,
        }
    }

    pub unsafe fn drop_table(table: &mut UnsafeTable<T>, width: usize) {
        unsafe { UnsafeArray::drop_array(&mut table.array, width, width); }
    }
}

pub(crate) struct CursorMut<'a, T> {
    current: OptNode<T>,
    table: &'a mut UnsafeTable<T>,
    column: usize,
    table_width: usize,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn current(&mut self) -> Option<&mut T> {
        if let Some(nn) = self.current.as_mut() {
            return Some(unsafe{nn.as_mut().value_mut()})
        }
        None
    }

    pub fn move_next(&mut self) {
        match self.current.take() {
            Some(current) => unsafe {
                self.current = current.as_ref().next();
            },
            None => {
                if self.column < self.table_width - 1 {
                    self.column += 1;
                    self.current = unsafe{self.table.array.index_mut(self.column)}.head();
                }
            }
        }
    }

    pub fn remove_current(&mut self) -> Option<T> {
        self.take_current_node().map(|nn| unsafe{Node::from_non_null(nn)}.into_value())
    }

    pub fn take_current_node(&mut self) -> OptNode<T> {
        let node = self.current;
        self.move_next();
        unsafe {
            LinkedList::<T>::take_node_from(self.table.array.index_mut(self.column), node)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::unsafe_array::UnsafeArray;

    struct A {}

    impl Drop for A {
        fn drop(&mut self) {
            println!("drop A")
        }
    }

    #[test]
    fn debug() {
        let mut array = UnsafeArray::<A>::uninit(20);
        unsafe { UnsafeArray::drop_array(&mut array, 0, 20); }
    }
}