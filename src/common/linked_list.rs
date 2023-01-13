use std::ptr::NonNull;
use std::mem;

pub(crate) type OptNode<T> = Option<NonNull<Node<T>>>;

pub(crate) struct Node<T> {
    prev: OptNode<T>,
    next: OptNode<T>,
    value: T,
}

impl<T> Node<T> {
    pub fn new(prev: OptNode<T>, next: OptNode<T>, value: T) -> Self {
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

    pub fn prev(&self) -> OptNode<T> {
        self.prev
    }

    pub fn next(&self) -> OptNode<T> {
        self.next
    }

    pub fn value(&mut self) -> &mut T {
        &mut self.value
    }

    pub unsafe fn from_non_null(non_null: NonNull<Self>) -> Self {
        *Box::from_raw(non_null.as_ptr())
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

        let mut node = unsafe{Node::from_non_null(option.unwrap())};
        self.head = mem::take(&mut node.next);
        Some(node.value)
    }

    pub fn cursor_front_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut { current: self.head, list: self }
    }

    pub(crate) fn push_node_front(&mut self, mut node: NonNull<Node<T>>) {
        unsafe {
            assert!(node.as_mut().prev.is_none());
            node.as_mut().next = match self.head {
                Some(mut non_null) => {
                    non_null.as_mut().prev = Some(node);
                    Some(non_null)
                }
                None=> None,
            }
        }
        self.head = Some(node);
    }

    pub(crate) fn pop_node_front(&mut self) -> OptNode<T> {
        let mut option = mem::take(&mut self.head);
        match option {
            Some(mut non_null) => {
                self.head = mem::take(&mut unsafe{non_null.as_mut().next});
                Some(non_null)
            },
            None => None
        }
    }

    pub(crate) unsafe fn remove_node(list: &mut LinkedList<T>, node: OptNode<T>) -> OptNode<T> {
        let mut node_non_null = node?;
        let prev = node_non_null.as_mut().prev;
        let next = node_non_null.as_mut().next;
        match prev {
            Some(mut non_null) => non_null.as_mut().next = next,
            None => { list.head = next; }
        }
        Some(node_non_null)
    }

    pub(crate) fn head(&self) -> OptNode<T> {
        self.head
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        LinkedList::new()
    }
}

pub(crate) struct CursorMut<'a, T> {
    current: OptNode<T>,
    list: &'a mut LinkedList<T>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn current(&mut self) -> Option<&mut T> {
        if let Some(non_null) = self.current.as_mut() {
            return Some(unsafe{non_null.as_mut().value()})
        }
        None
    }

    pub fn move_next(&mut self) {
        match self.current.take() {
            Some(current) => unsafe {
                self.current = current.as_ref().next;
            },
            None => {
                self.current = None;
            }
        }
    }

    pub fn remove_current(&mut self) -> Option<T> {
        let node = self.current;
        self.move_next();
        unsafe {
            LinkedList::<T>::remove_node(self.list, node)
                .map(|non_null| Node::from_non_null(non_null).into_value())
        }
    }
}
