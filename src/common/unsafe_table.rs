use std::ptr::NonNull;

use crate::common::linked_list::{LinkedList, Node, OptNode, CursorMut as ColumnCursorMut};
use crate::common::unsafe_array::UnsafeArray;

pub(crate) struct UnsafeTable<T> {
    array: UnsafeArray<LinkedList<T>>,
    // reserve: LinkedList<Option<T>>,
}

impl<T> UnsafeTable<T> {
    pub fn new(width: usize) -> Self {
        UnsafeTable {
            array: UnsafeArray::new(width, LinkedList::new)
        }
    }

    pub unsafe fn push_front(&mut self, column: usize, value: T) {
        self.array.index_mut(column).push_front(value);
    }

    pub unsafe fn pop_front(&mut self, column: usize) -> Option<T> {
        self.array.index_mut(column).pop_front()
    }

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
        unsafe { UnsafeArray::drop_array(&mut table.array, width); }
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




// pub(crate) struct CursorMut<'a, T> {
//     list_cursor: linked_list::CursorMut<'a, Option<T>>,
//     table: &'a mut UnsafeLinkedTable<T>,
//     table_width: usize,
//     current_list: usize,
// }

// impl<'a, T> CursorMut<'a, T> {
//     pub fn current(&mut self) -> Option<&mut T> {
//         match self.list_cursor.current() {
//             Some(opt) => opt.as_mut(),
//             None => None
//         }
//     }

//     pub fn move_next(&'a mut self) {
//         self.list_cursor.move_next();
//         if self.list_cursor.current().is_none() && self.current_list < (self.table_width - 1) {
//             self.current_list += 1;
//             let mut next_list = unsafe{self.table.array.index_mut(self.current_list)};
//             self.list_cursor = next_list.cursor_front_mut();
//         }
//     }

//     pub fn remove_current(&mut self) -> Option<T> {
//         self.list_cursor.remove_current()?
//         // match self.list_cursor.remove_current() {
//         //     Some(opt) => opt,
//         //     None => None
//         // }
//     }
// }


#[cfg(test)]
mod tests {
    #[test]
    fn debug() {}
}