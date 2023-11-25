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

    pub fn remove<T: 'static>(&mut self, id: &Id) -> T {
        let type_id = TypeId::of::<T>();
        assert!(type_id == id.type_id);

        let deque = self.data.get_mut(&type_id).unwrap();
        assert!(id.index < deque.len());

        unsafe { deque.try_remove(self.capacity, id.index).unwrap() }
    }

    pub fn remove_by_tid<T: 'static>(&mut self, id: &Tid<T>) -> T {
        let deque = self.data.get_mut(&TypeId::of::<T>()).unwrap();
        assert!(id.index < deque.len());

        unsafe { deque.try_remove(self.capacity, id.index).unwrap() }
    }

    pub fn get<T: 'static>(&self, id: &Id) -> &T {
        let type_id = TypeId::of::<T>();
        assert!(type_id == id.type_id);

        let deque = self.data.get(&type_id).unwrap();
        assert!(id.index < deque.len());

        unsafe { deque.try_get(self.capacity, id.index).unwrap() }
    }

    pub fn get_by_tid<T: 'static>(&self, id: &Tid<T>) -> &T {
        let deque = self.data.get(&TypeId::of::<T>()).unwrap();
        assert!(id.index < deque.len());

        unsafe { deque.try_get(self.capacity, id.index).unwrap() }
    }

    pub fn get_mut<T: 'static>(&mut self, id: &Id) -> &mut T {
        let type_id = TypeId::of::<T>();
        assert!(type_id == id.type_id);

        let deque = self.data.get_mut(&type_id).unwrap();
        assert!(id.index < deque.len());

        unsafe { deque.try_get_mut(self.capacity, id.index).unwrap() }
    }

    pub fn get_mut_by_tid<T: 'static>(&mut self, id: &Tid<T>) -> &mut T {
        let deque = self.data.get_mut(&TypeId::of::<T>()).unwrap();
        assert!(id.index < deque.len());

        unsafe { deque.try_get_mut(self.capacity, id.index).unwrap() }
    }

    pub fn shrink_to_fit(&mut self) {
        for (_, deque) in self.data.iter_mut() {
            unsafe {
                deque.shrink_to_fit(self.capacity);
            }
        }
        self.data.shrink_to_fit();
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
