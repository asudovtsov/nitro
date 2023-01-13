use crate::common::linked_list;
use crate::common::linked_list::LinkedList;
use crate::common::unsafe_array::UnsafeArray;

pub(crate) struct UnsafeLinkedTable<T> {
    array: UnsafeArray<LinkedList<Option<T>>>,
    reserve: LinkedList<Option<T>>,
}

impl<T> UnsafeLinkedTable<T> {
    pub fn with_capacity(width: usize, cell_capacity: usize) -> Self {
        assert_ne!(cell_capacity, 0); // #TODO implement grow_factor

        let mut reserve = LinkedList::new();
        for _ in 0..cell_capacity {
            reserve.push_front(None);
        }

        UnsafeLinkedTable {
            array: UnsafeArray::new_with(width, LinkedList::new),
            reserve,
        }
    }

    pub unsafe fn push_front(&mut self, column: usize, value: T) {
        self.array.index_mut(column).push_front(Some(value));
    }

    pub unsafe fn pop_front(&mut self, column: usize) -> Option<T> {
        self.array.index_mut(column).pop_front().flatten()
    }

    // cp means capacity-preserving
    pub unsafe fn push_front_cp(&mut self, column: usize, value: T) -> Result<(), T> {
        if let Some(mut non_null) = self.reserve.pop_node_front() {
            non_null.as_mut().value().replace(value);
            self.array.index_mut(column).push_node_front(non_null);
            return Ok(());
        }
        Err(value)
    }

    // cp means capacity-preserving
    pub unsafe fn pop_front_cp(&mut self, column: usize) -> Option<T> {
        match self.array.index_mut(column).pop_node_front() {
            Some(mut non_null) => {
                let value = std::mem::take(non_null.as_mut().value());
                self.reserve.push_node_front(non_null);
                value
            },
            None => None
        }
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

    pub unsafe fn drop_table(table: &mut UnsafeLinkedTable<T>, width: usize) {
        unsafe { UnsafeArray::drop_array(&mut table.array, width); }
    }
}

pub(crate) struct CursorMut<'a, T> {
    current: linked_list::OptNode<Option<T>>,
    table: &'a mut UnsafeLinkedTable<T>,
    column: usize,
    table_width: usize,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn current(&mut self) -> Option<&mut T> {
        if let Some(non_null) = self.current.as_mut() {
            return unsafe{non_null.as_mut().value().as_mut()}
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
        let node = self.current;
        self.move_next();
        unsafe {
            LinkedList::<Option<T>>::remove_node(self.table.array.index_mut(self.column), node).flatten()
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