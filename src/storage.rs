use crate::type_erased_deque::{BlockCapacity, TypeErasedDeque};
use core::any::TypeId;
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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

pub trait StorageId {
    fn index(&self) -> usize;
    fn type_id(&self) -> TypeId;
}

impl StorageId for Id {
    fn index(&self) -> usize {
        self.index
    }
    fn type_id(&self) -> TypeId {
        self.type_id
    }
}

impl<T: 'static> StorageId for Tid<T> {
    fn index(&self) -> usize {
        self.index
    }
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
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

    pub fn remove<T: 'static>(&mut self, id: &impl StorageId) -> Option<T> {
        let capacity = self.capacity;
        self.for_deque_mut::<T, _, _>(id.index(), &id.type_id(), None, |deque| unsafe {
            deque.try_remove(capacity, id.index())
        })
    }

    pub fn get<T: 'static>(&self, id: &impl StorageId) -> Option<&T> {
        let capacity = self.capacity;
        self.for_deque::<T, _, _>(id.index(), &id.type_id(), None, |deque| unsafe {
            deque.try_get(capacity, id.index())
        })
    }

    pub fn get_mut<T: 'static>(&mut self, id: &impl StorageId) -> Option<&mut T> {
        let capacity = self.capacity;
        self.for_deque_mut::<T, _, _>(id.index(), &id.type_id(), None, |deque| unsafe {
            deque.try_get_mut(capacity, id.index())
        })
    }

    pub fn contains(&self, id: &Id) -> bool {
        match self.data.get(&id.type_id) {
            Some(deque) => deque.contains(id.index),
            None => false,
        }
    }

    pub fn shrink_to_fit(&mut self) {
        for (_, deque) in self.data.iter_mut() {
            unsafe {
                deque.shrink_to_fit(self.capacity);
            }
        }
        self.data.shrink_to_fit();
    }

    fn for_deque<'a, T, R, F>(&'a self, index: usize, type_id: &TypeId, default: R, f: F) -> R
    where
        F: Fn(&'a TypeErasedDeque) -> R,
        R: 'a,
        T: 'static,
    {
        if *type_id != TypeId::of::<T>() {
            return default;
        }

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

    fn for_deque_mut<'a, T, R, F>(
        &'a mut self,
        index: usize,
        type_id: &TypeId,
        default: R,
        f: F,
    ) -> R
    where
        F: Fn(&'a mut TypeErasedDeque) -> R,
        R: 'a,
        T: 'static,
    {
        if *type_id != TypeId::of::<T>() {
            return default;
        }

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
