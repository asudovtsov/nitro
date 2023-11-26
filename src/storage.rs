use crate::bucket::{BlockCapacity, Bucket};
use core::any::TypeId;
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Id<T> {
    index: usize,
    cycle: u64,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Id<T> {
    fn new(inbucket_index: usize, cycle: u64) -> Self {
        Self {
            index: inbucket_index,
            cycle,
            phantom: Default::default(),
        }
    }
}

pub struct Storage {
    data: HashMap<TypeId, Bucket>,
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

    pub fn place<T: 'static>(&mut self, data: T) -> Id<T> {
        let (index, cycle) = unsafe {
            self.data
                .entry(TypeId::of::<T>())
                .or_insert(Bucket::new::<T>())
                .try_place(self.capacity, data)
                .unwrap()
        };

        Id::new(index, cycle)
    }

    pub fn remove<T: 'static>(&mut self, id: &Id<T>) -> Option<T> {
        match self.data.get_mut(&TypeId::of::<T>()) {
            Some(bucket) if id.index < bucket.len() => unsafe {
                bucket.try_remove(self.capacity, id.index)
            },
            _ => None,
        }
    }

    pub fn get<T: 'static>(&self, id: &Id<T>) -> Option<&T> {
        match self.data.get(&TypeId::of::<T>()) {
            Some(bucket) if id.index < bucket.len() => unsafe {
                bucket.try_get(self.capacity, id.index)
            },
            _ => None,
        }
    }

    pub fn get_mut<T: 'static>(&mut self, id: &Id<T>) -> Option<&mut T> {
        match self.data.get_mut(&TypeId::of::<T>()) {
            Some(bucket) if id.index < bucket.len() => unsafe {
                bucket.try_get_mut(self.capacity, id.index)
            },
            _ => None,
        }
    }

    pub fn contains<T: 'static>(&self, id: &Id<T>) -> bool {
        match self.data.get(&TypeId::of::<T>()) {
            Some(bucket) => bucket.contains(id.index),
            None => false,
        }
    }

    pub fn dead_cells_count<T: 'static>(&self) -> usize {
        match self.data.get(&TypeId::of::<T>()) {
            Some(bucket) => bucket.dead_count(),
            None => 0,
        }
    }

    pub fn shrink_to_fit(&mut self) {
        for (_, bucket) in self.data.iter_mut() {
            unsafe {
                bucket.shrink_to_fit(self.capacity);
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
        for (_, bucket) in self.data.iter_mut() {
            unsafe { Bucket::drop(bucket, self.capacity) }
        }
    }
}
