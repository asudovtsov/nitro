use crate::{
    bucket::Bucket,
    tag::{Unique32, UniqueTag},
};
use core::any::TypeId;
use std::collections::HashMap;

pub trait AsTid<T, U: UniqueTag> {
    fn as_tid(&self) -> Option<&Tid<T, U>>;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Tid<T, U: UniqueTag = Unique32> {
    index: usize,
    tag: U,
    phantom: std::marker::PhantomData<T>,
}

impl<T, U: UniqueTag> Tid<T, U> {
    fn new(inbucket_index: usize, tag: U) -> Self {
        Self {
            index: inbucket_index,
            tag,
            phantom: Default::default(),
        }
    }
}

impl<T, U: UniqueTag> AsTid<T, U> for &Tid<T, U> {
    fn as_tid(&self) -> Option<&Tid<T, U>> {
        Some(self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Id<U: UniqueTag = Unique32> {
    tid: Tid<(), U>,
    type_id: TypeId,
}

impl<U: UniqueTag> Id<U> {
    fn new(inbucket_index: usize, tag: U, type_id: TypeId) -> Self {
        Self {
            tid: Tid::new(inbucket_index, tag),
            type_id,
        }
    }

    fn index(&self) -> usize {
        self.tid.index
    }
}

impl<T: 'static, U: UniqueTag> From<Tid<T, U>> for Id<U> {
    fn from(value: Tid<T, U>) -> Self {
        Self {
            tid: Tid::new(value.index, value.tag),
            type_id: TypeId::of::<T>(),
        }
    }
}

impl<T: 'static, U: UniqueTag> AsTid<T, U> for &Id<U> {
    fn as_tid(&self) -> Option<&Tid<T, U>> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        Some(unsafe { std::mem::transmute(&self.tid) })
    }
}

pub struct Storage<U: UniqueTag = Unique32> {
    data: HashMap<TypeId, Bucket<U>>,
    capacity: usize,
}

impl Storage<Unique32> {
    pub fn new() -> Self {
        Self::with_block_capacity(1024)
    }

    pub fn with_block_capacity(capacity: usize) -> Self {
        Storage::new_with_tag_and_block_capacity::<Unique32>(capacity)
    }
}

impl Storage {
    pub fn new_with_tag<U: UniqueTag>() -> Storage<U> {
        Self::new_with_tag_and_block_capacity(1024)
    }

    pub fn new_with_tag_and_block_capacity<U: UniqueTag>(capacity: usize) -> Storage<U> {
        Storage {
            data: Default::default(),
            capacity,
        }
    }
}

impl<U: UniqueTag> Storage<U> {
    pub fn place<T: 'static>(&mut self, data: T) -> Tid<T, U> {
        let (index, tag) = unsafe {
            self.data
                .entry(TypeId::of::<T>())
                .or_insert(Bucket::new::<T>())
                .place_unchecked(self.capacity, data)
        };

        Tid::new(index, tag)
    }

    pub fn place_id<T: 'static>(&mut self, data: T) -> Id<U> {
        let type_id = TypeId::of::<T>();
        let (index, tag) = unsafe {
            self.data
                .entry(type_id)
                .or_insert(Bucket::new::<T>())
                .place_unchecked(self.capacity, data)
        };

        Id::new(index, tag, type_id)
    }

    pub fn remove<T: 'static>(&mut self, id: impl AsTid<T, U>) -> Option<T> {
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

    pub fn get<T: 'static>(&self, id: impl AsTid<T, U>) -> Option<&T> {
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

    pub fn get_mut<T: 'static>(&mut self, id: impl AsTid<T, U>) -> Option<&mut T> {
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

    pub fn contains<T: 'static>(&self, id: impl AsTid<T, U>) -> bool {
        match id.as_tid() {
            Some(tid) => match self.data.get(&TypeId::of::<T>()) {
                Some(bucket) => bucket.contains(self.capacity, tid.index),
                None => false,
            },
            None => false,
        }
    }

    pub fn contains_id(&self, id: &Id<U>) -> bool {
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

    // remove all placed data, reset all unique tags, reset all banned cells
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

impl<U: UniqueTag> Drop for Storage<U> {
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

        let mut storage = Storage::new_with_tag::<crate::tag::NoTag>();
        // let red = storage.place((String::from("red"), 255, 0, 0));
        // let green = storage.place((String::from("green"), 0, 255, 0));

        for i in 0..10_000_000 {
            let index = storage.place::<Color>((String::from("red"), 255, 0, 0));
            // storage.remove(&index);
            if i % 10_000_000 - 1 == 0 {
                println!(
                    "! {} {:?} {}",
                    i,
                    index,
                    storage.contains(&index),
                    // storage.dead_cells_count::<Color>()
                );
            }
            // let id = storage.place_id::<Color>((String::from("red"), 255, 0, 0));
            // storage.contains_id(&id);
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
