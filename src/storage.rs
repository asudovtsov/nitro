use crate::{
    bucket::{BlockCapacity, Bucket},
    cycle::{Blocking32, Cycle},
};
use core::any::TypeId;
use std::collections::HashMap;

pub trait AsTid<T, C: Cycle> {
    fn as_tid(&self) -> Option<&Tid<T, C>>;
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Tid<T, C: Cycle = Blocking32> {
    index: usize,
    cycle: C,
    phantom: std::marker::PhantomData<T>,
}

impl<T, C: Cycle> Copy for Tid<T, C> {}

impl<T, C: Cycle> Clone for Tid<T, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, C: Cycle> Tid<T, C> {
    fn new(inbucket_index: usize, cycle: C) -> Self {
        Self {
            index: inbucket_index,
            cycle,
            phantom: Default::default(),
        }
    }
}

impl<T, C: Cycle> AsTid<T, C> for &Tid<T, C> {
    fn as_tid(&self) -> Option<&Tid<T, C>> {
        Some(self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Id<C: Cycle = Blocking32> {
    tid: Tid<(), C>,
    type_id: TypeId,
}

impl<C: Cycle> Id<C> {
    fn new(inbucket_index: usize, cycle: C, type_id: TypeId) -> Self {
        Self {
            tid: Tid::new(inbucket_index, cycle),
            type_id,
        }
    }

    fn index(&self) -> usize {
        self.tid.index
    }
}

impl<T: 'static, C: Cycle> From<Tid<T, C>> for Id<C> {
    fn from(value: Tid<T, C>) -> Self {
        Self {
            tid: Tid::new(value.index, value.cycle),
            type_id: TypeId::of::<T>(),
        }
    }
}

impl<T: 'static, C: Cycle> AsTid<T, C> for &Id<C> {
    fn as_tid(&self) -> Option<&Tid<T, C>> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        Some(unsafe { std::mem::transmute(&self.tid) })
    }
}

pub struct Storage<C: Cycle = Blocking32> {
    data: HashMap<TypeId, Bucket<C>>,
    capacity: BlockCapacity,
}

impl Storage<Blocking32> {
    pub fn new() -> Self {
        Self::with_block_capacity(1024)
    }

    pub fn with_block_capacity(capacity: usize) -> Self {
        Storage::new_with_cycle_and_block_capacity::<Blocking32>(capacity)
    }
}

impl Storage {
    pub fn new_with_cycle<A: Cycle>() -> Storage<A> {
        Self::new_with_cycle_and_block_capacity(1024)
    }

    pub fn new_with_cycle_and_block_capacity<A: Cycle>(capacity: usize) -> Storage<A> {
        Storage {
            data: Default::default(),
            capacity: BlockCapacity::new(capacity),
        }
        }
    }

impl<C: Cycle> Storage<C> {
    pub fn place<T: 'static>(&mut self, data: T) -> Tid<T, C> {
        let (index, cycle) = unsafe {
            self.data
                .entry(TypeId::of::<T>())
                .or_insert(Bucket::new::<T>())
                .place_unchecked(self.capacity, data)
        };

        Tid::new(index, cycle)
    }

    pub fn place_id<T: 'static>(&mut self, data: T) -> Id<C> {
        let type_id = TypeId::of::<T>();
        let (index, cycle) = unsafe {
            self.data
                .entry(type_id)
                .or_insert(Bucket::new::<T>())
                .place_unchecked(self.capacity, data)
        };

        Id::new(index, cycle, type_id)
    }

    pub fn remove<T: 'static>(&mut self, id: impl AsTid<T, C>) -> Option<T> {
        match id.as_tid() {
            Some(tid) => match self.data.get_mut(&TypeId::of::<T>()) {
                Some(bucket) if tid.index < bucket.len() => unsafe {
                    bucket.remove_unchecked(self.capacity, tid.index)
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn get<T: 'static>(&self, id: impl AsTid<T, C>) -> Option<&T> {
        match id.as_tid() {
            Some(tid) => match self.data.get(&TypeId::of::<T>()) {
                Some(bucket) if tid.index < bucket.len() => unsafe {
                    bucket.get_ucnhecked(self.capacity, tid.index)
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn get_mut<T: 'static>(&mut self, id: impl AsTid<T, C>) -> Option<&mut T> {
        match id.as_tid() {
            Some(tid) => match self.data.get_mut(&TypeId::of::<T>()) {
                Some(bucket) if tid.index < bucket.len() => unsafe {
                    bucket.get_mut_unchecked(self.capacity, tid.index)
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn contains<T: 'static>(&self, id: impl AsTid<T, C>) -> bool {
        match id.as_tid() {
            Some(tid) => match self.data.get(&TypeId::of::<T>()) {
                Some(bucket) => bucket.contains(self.capacity, tid.index),
                None => false,
            },
            None => false,
        }
    }

    pub fn contains_id(&self, id: &Id<C>) -> bool {
        match self.data.get(&id.type_id) {
            Some(bucket) => bucket.contains(self.capacity, id.index()),
            None => false,
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

    // remove all placed data
    pub fn clear(&mut self) {
        for (_, bucket) in self.data.iter_mut() {
            unsafe {
                Bucket::clear(bucket, self.capacity);
            }
        }
    }

    // remove all placed data, reset all cycles, reset all banned cells
    pub fn reset(&mut self) {
        for (_, bucket) in self.data.iter_mut() {
            unsafe {
                Bucket::reset(bucket, self.capacity);
            }
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Cycle> Drop for Storage<C> {
    fn drop(&mut self) {
        for (_, bucket) in self.data.iter_mut() {
            unsafe { Bucket::drop(bucket, self.capacity) }
        }
    }
}
