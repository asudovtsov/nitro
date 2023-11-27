use crate::bucket::{BlockCapacity, Bucket};
use core::any::TypeId;
use std::collections::HashMap;

pub trait AsTid<T> {
    fn as_tid(&self) -> Option<&Tid<T>>;
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Tid<T> {
    index: usize,
    cycle: u64,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Copy for Tid<T> {}

impl<T> Clone for Tid<T> {
    fn clone(&self) -> Self {
        *self
    }
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

impl<T> AsTid<T> for &Tid<T> {
    fn as_tid(&self) -> Option<&Tid<T>> {
        Some(self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Id {
    tid: Tid<()>,
    type_id: TypeId,
}

impl Id {
    fn new(inbucket_index: usize, cycle: u64, type_id: TypeId) -> Self {
        Self {
            tid: Tid::new(inbucket_index, cycle),
            type_id,
        }
    }

    fn index(&self) -> usize {
        self.tid.index
    }
}

impl<T: 'static> From<Tid<T>> for Id {
    fn from(value: Tid<T>) -> Self {
        Self {
            tid: Tid::new(value.index, value.cycle),
            type_id: TypeId::of::<T>(),
        }
    }
}

impl<T: 'static> AsTid<T> for &Id {
    fn as_tid(&self) -> Option<&Tid<T>> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        Some(unsafe { std::mem::transmute(&self.tid) })
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

    pub fn remove<T: 'static>(&mut self, id: impl AsTid<T>) -> Option<T> {
        match id.as_tid() {
            Some(tid) => match self.data.get_mut(&TypeId::of::<T>()) {
                Some(bucket) if tid.index < bucket.len() => unsafe {
                    bucket.try_remove(self.capacity, tid.index)
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn get<T: 'static>(&self, id: impl AsTid<T>) -> Option<&T> {
        match id.as_tid() {
            Some(tid) => match self.data.get(&TypeId::of::<T>()) {
                Some(bucket) if tid.index < bucket.len() => unsafe {
                    bucket.try_get(self.capacity, tid.index)
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn get_mut<T: 'static>(&mut self, id: impl AsTid<T>) -> Option<&mut T> {
        match id.as_tid() {
            Some(tid) => match self.data.get_mut(&TypeId::of::<T>()) {
                Some(bucket) if tid.index < bucket.len() => unsafe {
                    bucket.try_get_mut(self.capacity, tid.index)
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn contains<T: 'static>(&self, id: impl AsTid<T>) -> bool {
        match id.as_tid() {
            Some(tid) => match self.data.get(&TypeId::of::<T>()) {
                Some(bucket) => bucket.contains(tid.index),
                None => false,
            },
            None => false,
        }
    }

    pub fn contains_id(&self, id: &Id) -> bool {
        match self.data.get(&id.type_id) {
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

// #[derive(Debug)]
// struct A {
//     string: String,
// }

// impl Drop for A {
//     fn drop(&mut self) {
//         println!("###Drop A {:?}", self);
//     }
// }

mod tests {
    #[test]
    fn place_remove_contains() {
        use super::*;

        type Color = (String, u8, u8, u8);
        // struct Label(String);

        let mut storage = Storage::new();
        let red = storage.place((String::from("red"), 255, 0, 0));
        let green = storage.place((String::from("green"), 0, 255, 0));

        for i in 0..10_000_000 {
            let index = storage.place::<Color>((String::from("red"), 255, 0, 0));
            // storage.remove(&index);
            if i % 10_000_000 - 1 == 0 {
                println!(
                    "! {} {:?} {} {}",
                    i,
                    index,
                    storage.contains(&index),
                    storage.dead_cells_count::<Color>()
                );
            }
            let id = storage.place_id::<Color>((String::from("red"), 255, 0, 0));
            storage.contains_id(&id);
        }

        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf);

        // let blue = storage.place((String::from("blue"), 0, 0, 255));
        // let label = storage.place(Label("label".into()));
        // println!("{:?}\n{:?}\n{:?}", red, green, blue);
        // println!(
        //     "{:?}\n{:?}\n{:?}",
        //     storage.contains(&red),
        //     storage.contains(&green),
        //     storage.contains(&blue)
        // );

        // assert!(storage.contains::<Color>(&red));
        // assert!(storage.contains::<Color>(&green));
        // assert!(storage.contains::<Color>(&blue));
        // assert!(storage.contains::<Label>(&label));
    }
}
