use std::ptr::{NonNull, addr_of_mut};
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
pub(crate) struct LinkedList<T> {
    head: OptNode<T>
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn head(&self) -> OptNode<T> {
        self.head
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

    pub fn push_node_front(&mut self, mut node: NonNull<Node<T>>) {
        unsafe {
            assert!(node.as_mut().prev.is_none());
            node.as_mut().next = match self.head {
                Some(mut nn) => {
                    nn.as_mut().prev = Some(node);
                    Some(nn)
                }
                None=> None,
            }
        }
        self.head = Some(node);
    }

    pub fn pop_node_front(&mut self) -> OptNode<T> {
        let option = mem::take(&mut self.head);
        match option {
            Some(mut nn) => {
                self.head = mem::take(&mut unsafe{nn.as_mut().next});
                Some(nn)
            },
            None => None
        }
    }

    pub fn cursor_front_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut { current: self.head, list: self }
    }

    pub fn unsafe_picker(&mut self, node: OptNode<T>) -> UnsafePicker<T> {
        UnsafePicker {
            node: node,
            list: unsafe {NonNull::new_unchecked(addr_of_mut!(*self))}
        }
    }

    pub unsafe fn take_node_from(list: &mut LinkedList<T>, node: OptNode<T>) -> OptNode<T> {
        let mut node_nn = node?;
        let prev = node_nn.as_mut().prev;
        let next = node_nn.as_mut().next;
        match prev {
            Some(mut nn) => nn.as_mut().next = next,
            None => { list.head = next; }
        }
        Some(node_nn)
    }

    pub fn leak(self) {
        self.head = None;
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut opt_node = self.head;
        while let Some(mut nn) = opt_node {
            unsafe {
                Node::from_non_null(nn);
                opt_node = nn.as_mut().next;
            };
        }
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
        if let Some(nn) = self.current.as_mut() {
            return Some(unsafe{nn.as_mut().value_mut()})
        }
        None
    }

    pub fn current_node(&self) -> OptNode<T> {
        self.current
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
            LinkedList::<T>::take_node_from(self.list, node)
                .map(|nn| Node::from_non_null(nn).into_value())
        }
    }

    pub fn replace_current_node(&mut self, nn: NonNull<Node<T>>) -> OptNode<T> {
        debug_assert!(self.current.is_some());
        todo!()
    }
}

pub(crate) struct UnsafePicker<T> {
    node: OptNode<T>,
    list: NonNull<LinkedList<T>>,
}

impl<'a, T> UnsafePicker<T> {
    pub unsafe fn pick(&mut self) -> T {
        let nn = LinkedList::<T>::take_node_from(self.list.as_mut(), self.node).unwrap_unchecked();
        Node::from_non_null(nn).into_value()
    }

    pub unsafe fn pick_mut(&mut self) -> &mut T {
        self.node.as_mut().unwrap_unchecked().as_mut().value_mut()
    }

    pub unsafe fn pick_node(&mut self) -> OptNode<T> {
        LinkedList::<T>::take_node_from(self.list.as_mut(), self.node)
    }
}
