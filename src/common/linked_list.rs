use std::ptr::{NonNull, addr_of_mut, null_mut};
use std::mem;

pub(crate) type OptNode<T> = Option<NonNull<Node<T>>>;

pub(crate) struct Node<T> {
    prev: *mut Node<T>,
    next: *mut Node<T>,
    value: T,
}

impl<T> Node<T> {
    pub fn new(prev: *mut Node<T>, next: *mut Node<T>, value: T) -> Self {
        Node {
            prev,
            next,
            value
        }
    }

    pub fn unwrap(self) -> T {
        self.value
    }

    pub fn into_value_from_box(self: Box<Self>) -> T {
        self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn prev(&self) -> *mut Node<T> {
        self.prev
    }

    pub fn next(&self) -> *mut Node<T> {
        self.next
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn value_ref(&self) -> &T {
        &self.value
    }

    pub unsafe fn from_non_null(nn: NonNull<Self>) -> Self {
        *Box::from_raw(nn.as_ptr())
    }
}

#[derive(Clone)]
pub(crate) struct List<T> {
    head: *mut Node<T>
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: null_mut()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    // pub fn head(&self) -> *mut Node<T> {
    //     self.head
    // }

    pub fn push_node_front(&mut self, mut node: *mut Node<T>) {
        assert!(!node.is_null());

        let mut old_head = self.head;
        self.head = node;

        if !old_head.is_null() {
            unsafe {
                (*old_head).prev = node;
                (*node).next = old_head;
            }
        }
    }

    pub fn pop_node_front(&mut self) -> *mut Node<T> {
        let mut old_head = self.head;
        if old_head.is_null() {
            return null_mut();
        }

        unsafe {
            (*old_head).prev = null_mut();
            self.head = (*old_head).next;
        }

        old_head
    }

    pub fn cursor_front_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut { current: self.head, list: self }
    }

    // pub fn unsafe_picker(&mut self, node: OptNode<T>) -> UnsafePicker<T> {
    //     UnsafePicker {
    //         node: node,
    //         list: unsafe {NonNull::new_unchecked(addr_of_mut!(*self))}
    //     }
    // }

    pub unsafe fn take_node_from(list: &mut List<T>, node: *mut Node<T>) {
        assert!(!node.is_null());

        let prev = unsafe{&*node}.prev;
        let next = unsafe{&*node}.next;

        if prev.is_null() {
            list.head = next;
        } else {
            unsafe{&mut *prev}.next = next;
        }
    }

    // pub fn leak(self) {
    //     self.head = None;
    // }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        todo!()
        // let mut opt_node = self.head;
        // while let Some(mut nn) = opt_node {
        //     unsafe {
        //         Node::from_non_null(nn);
        //         opt_node = nn.as_mut().next;
        //     };
        // }
    }
}

// impl<T> Default for LinkedList<T> {
//     fn default() -> Self {
//         LinkedList::new()
//     }
// }

pub(crate) struct CursorMut<'a, T> {
    current: *mut Node<T>,
    list: &'a mut List<T>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn current(&mut self) -> Option<&mut T> {
        if !self.current.is_null() {
            return Some(&mut unsafe{&mut *self.current}.value)
        }
        None
    }

    pub fn current_node(&self) -> *mut Node<T> {
        self.current
    }

    pub fn move_next(&mut self) -> bool {
        if !self.current.is_null() {
            self.current = unsafe{&*self.current}.next;
            return true
        }
        false
    }

    pub fn remove_current(&mut self) -> Option<T> {
        let node = self.current;
        if node.is_null() {
            return None;
        }

        self.move_next();
        unsafe {
            List::<T>::take_node_from(self.list, node);
            Some(Box::from_raw(node).into_value_from_box())
        }
    }

    // pub fn replace_current_node(&mut self, nn: NonNull<Node<T>>) -> OptNode<T> {
    //     debug_assert!(!self.current.is_null());
    //     todo!()
    // }
}

// pub(crate) struct UnsafePicker<T> {
//     node: OptNode<T>,
//     list: NonNull<LinkedList<T>>,
// }

// impl<'a, T> UnsafePicker<T> {
//     pub unsafe fn pick(&mut self) -> T {
//         let nn = LinkedList::<T>::take_node_from(self.list.as_mut(), self.node).unwrap_unchecked();
//         Node::from_non_null(nn).into_value()
//     }

//     pub unsafe fn pick_mut(&mut self) -> &mut T {
//         self.node.as_mut().unwrap_unchecked().as_mut().value_mut()
//     }

//     pub unsafe fn pick_node(&mut self) -> OptNode<T> {
//         LinkedList::<T>::take_node_from(self.list.as_mut(), self.node)
//     }
// }
