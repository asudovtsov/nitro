use crate::type_erased_deque::{BlockCapacity, TypeErasedDeque};
use core::any::TypeId;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Id {
    index: usize,
    type_id: TypeId,
}

impl Id {
    pub fn new(index: usize, type_id: TypeId) -> Self {
        Self { index, type_id }
    }

    pub fn new_for<T: 'static>(index: usize) -> Self {
        Self {
            index,
            type_id: TypeId::of::<T>(),
        }
    }
}

pub struct Tid<T> {
    index: usize,
    phantom: core::marker::PhantomData<T>,
}

impl<T> Tid<T> {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            phantom: Default::default(),
        }
    }
}

impl<T: 'static> core::fmt::Debug for Tid<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Tid")
            .field("index", &self.index)
            .field("type_id", &TypeId::of::<T>())
            .finish()
    }
}

pub struct Storage {
    data: HashMap<TypeId, TypeErasedDeque>,
    capacity: BlockCapacity,
}

impl Storage {
    pub fn new() -> Self {
        Self::with_block_capacity(1024)
    }

    pub fn with_block_capacity(capacity: usize) -> Self {
        Self {
            data: Default::default(),
            capacity: BlockCapacity::new(capacity),
        }
    }

    pub fn place<T: 'static>(&mut self, data: T) -> Id {
        let type_id = TypeId::of::<T>();
        let index = unsafe {
            self.data
                .entry(type_id)
                .or_insert(TypeErasedDeque::new::<T>())
                .try_push(self.capacity, data)
                .unwrap()
        };

        Id::new(index, type_id)
    }

    pub fn place_with_tid<T: 'static>(&mut self, data: T) -> Tid<T> {
        let type_id = TypeId::of::<T>();
        let index = unsafe {
            self.data
                .entry(type_id)
                .or_insert(TypeErasedDeque::new::<T>())
                .try_push(self.capacity, data)
                .unwrap()
        };

        Tid::new(index)
    }

    pub fn remove<T: 'static>(&mut self, id: &Id) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if type_id != id.type_id {
            return None;
        }

        let capacity = self.capacity;
        self.for_deque_mut(id.index, &type_id, None, |deque| unsafe {
            deque.try_remove::<T>(capacity, id.index)
        })
    }

    pub fn remove_by_tid<T: 'static>(&mut self, id: &Tid<T>) -> Option<T> {
        let capacity = self.capacity;
        self.for_deque_mut(id.index, &TypeId::of::<T>(), None, |deque| unsafe {
            deque.try_remove::<T>(capacity, id.index)
        })
    }

    pub fn get<T: 'static>(&self, id: &Id) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if type_id != id.type_id {
            return None;
        }

        let capacity = self.capacity;
        self.for_deque(id.index, &type_id, None, |deque| unsafe {
            deque.try_get(capacity, id.index)
        })
    }

    pub fn get_by_tid<T: 'static>(&self, id: &Tid<T>) -> Option<&T> {
        let capacity = self.capacity;
        self.for_deque(id.index, &TypeId::of::<T>(), None, |deque| unsafe {
            deque.try_get(capacity, id.index)
        })
    }

    pub fn get_mut<T: 'static>(&mut self, id: &Id) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if type_id != id.type_id {
            return None;
        }

        let capacity = self.capacity;
        self.for_deque_mut(id.index, &type_id, None, |deque| unsafe {
            deque.try_get_mut(capacity, id.index)
        })
    }

    pub fn get_mut_by_tid<T: 'static>(&mut self, id: &Tid<T>) -> Option<&mut T> {
        let capacity = self.capacity;
        self.for_deque_mut(id.index, &TypeId::of::<T>(), None, |deque| unsafe {
            deque.try_get_mut(capacity, id.index)
        })
    }

    pub fn shrink_to_fit(&mut self) {
        for (_, deque) in self.data.iter_mut() {
            unsafe {
                deque.shrink_to_fit(self.capacity);
            }
        }
        self.data.shrink_to_fit();
    }

    fn for_deque<'a, R, F>(&'a self, index: usize, type_id: &TypeId, default: R, f: F) -> R
    where
        F: Fn(&'a TypeErasedDeque) -> R,
        R: 'a,
    {
        match self.data.get(type_id) {
            Some(deque) => {
                if index >= deque.len() {
                    return default;
                }

                f(deque)
            }
            None => default,
        }
    }

    fn for_deque_mut<'a, R, F>(&'a mut self, index: usize, type_id: &TypeId, default: R, f: F) -> R
    where
        F: Fn(&'a mut TypeErasedDeque) -> R,
        R: 'a,
    {
        match self.data.get_mut(type_id) {
            Some(deque) => {
                if index >= deque.len() {
                    return default;
                }

                f(deque)
            }
            None => default,
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        for (_, deque) in self.data.iter_mut() {
            unsafe { TypeErasedDeque::drop(deque, self.capacity) }
        }
    }
}
