use crate::bucket::{BlockCapacity, Bucket};
use core::any::TypeId;
use std::collections::HashMap;

pub struct TypeIdHook<T>(TypeId, std::marker::PhantomData<T>);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Tid<T> {
    index: usize,
    cycle: u64,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Tid<T> {
    fn new(inbucket_index: usize, cycle: u64) -> Self {
        Self {
            index: inbucket_index,
            cycle,
            phantom: Default::default(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Id {
    index: usize,
    cycle: u64,
    type_id: TypeId,
}

impl Id {
    fn new(index: usize, cycle: u64, type_id: TypeId) -> Self {
        Self {
            index,
            cycle,
            type_id,
        }
    }
}

impl<T: 'static> From<Tid<T>> for Id {
    fn from(value: Tid<T>) -> Self {
        Self {
            index: value.index,
            cycle: value.cycle,
            type_id: TypeId::of::<T>(),
        }
    }
}

pub trait StoragerId<T: 'static> {
    fn cycle(&self) -> u64;
    fn index(&self) -> usize;
    fn type_id_hook(&self) -> TypeIdHook<T>;
}

impl<T: 'static> StoragerId<T> for Tid<T> {
    fn cycle(&self) -> u64 {
        self.cycle
    }
    fn index(&self) -> usize {
        self.index
    }
    fn type_id_hook(&self) -> TypeIdHook<T> {
        TypeIdHook(TypeId::of::<T>(), Default::default())
    }
}

impl<T: 'static> StoragerId<T> for Id {
    fn cycle(&self) -> u64 {
        self.cycle
    }
    fn index(&self) -> usize {
        self.index
    }
    fn type_id_hook(&self) -> TypeIdHook<T> {
        TypeIdHook(self.type_id, Default::default())
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

    pub fn place<T: 'static>(&mut self, data: T) -> Tid<T> {
        let (index, cycle) = unsafe {
            self.data
                .entry(TypeId::of::<T>())
                .or_insert(Bucket::new::<T>())
                .try_place(self.capacity, data)
                .unwrap()
        };

        Tid::new(index, cycle)
    }

    pub fn place_id<T: 'static>(&mut self, data: T) -> Id {
        let type_id = TypeId::of::<T>();
        let (index, cycle) = unsafe {
            self.data
                .entry(type_id)
                .or_insert(Bucket::new::<T>())
                .try_place(self.capacity, data)
                .unwrap()
        };

        Id::new(index, cycle, type_id)
    }

    pub fn remove<T: 'static>(&mut self, id: &dyn StoragerId<T>) -> Option<T> {
        match self.data.get_mut(&id.type_id_hook().0) {
            Some(bucket) if id.index() < bucket.len() => unsafe {
                bucket.try_remove(self.capacity, id.index())
            },
            _ => None,
        }
    }

    pub fn get<T: 'static>(&self, id: &dyn StoragerId<T>) -> Option<&T> {
        match self.data.get(&id.type_id_hook().0) {
            Some(bucket) if id.index() < bucket.len() => unsafe {
                bucket.try_get(self.capacity, id.index())
            },
            _ => None,
        }
    }

    pub fn get_mut<T: 'static>(&mut self, id: &dyn StoragerId<T>) -> Option<&mut T> {
        match self.data.get_mut(&id.type_id_hook().0) {
            Some(bucket) if id.index() < bucket.len() => unsafe {
                bucket.try_get_mut(self.capacity, id.index())
            },
            _ => None,
        }
    }

    pub fn contains<T: 'static>(&self, id: &dyn StoragerId<T>) -> bool {
        match self.data.get(&id.type_id_hook().0) {
            Some(bucket) => bucket.contains(id.index()),
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
