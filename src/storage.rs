use crate::{
    bucket::Bucket,
    gen::{Gen, Unique32},
};
use core::any::TypeId;
use std::collections::HashMap;

pub trait AsTid<T, G: Gen> {
    fn as_tid(&self) -> Option<&Tid<T, G>>;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Tid<T, G: Gen = Unique32> {
    index: usize,
    gen: G,
    phantom: std::marker::PhantomData<T>,
}

impl<T, G: Gen> Tid<T, G> {
    fn new(inbucket_index: usize, gen: G) -> Self {
        Self {
            index: inbucket_index,
            gen,
            phantom: Default::default(),
        }
    }
}

impl<T, G: Gen> AsTid<T, G> for &Tid<T, G> {
    fn as_tid(&self) -> Option<&Tid<T, G>> {
        Some(self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Id<G: Gen = Unique32> {
    tid: Tid<(), G>,
    type_id: TypeId,
}

impl<G: Gen> Id<G> {
    fn new(inbucket_index: usize, gen: G, type_id: TypeId) -> Self {
        Self {
            tid: Tid::new(inbucket_index, gen),
            type_id,
        }
    }

    fn index(&self) -> usize {
        self.tid.index
    }
}

impl<T: 'static, G: Gen> From<Tid<T, G>> for Id<G> {
    fn from(value: Tid<T, G>) -> Self {
        Self {
            tid: Tid::new(value.index, value.gen),
            type_id: TypeId::of::<T>(),
        }
    }
}

impl<T: 'static, G: Gen> AsTid<T, G> for &Id<G> {
    fn as_tid(&self) -> Option<&Tid<T, G>> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        Some(unsafe { std::mem::transmute(&self.tid) })
    }
}

pub struct Storage<G: Gen = Unique32> {
    data: HashMap<TypeId, Bucket<G>>,
    capacity: usize,
}

impl Storage<Unique32> {
    pub fn new() -> Self {
        Self::with_block_capacity(1024)
    }

    pub fn with_block_capacity(capacity: usize) -> Self {
        Storage::new_with_gen_and_block_capacity::<Unique32>(capacity)
    }
}

impl Storage {
    pub fn new_with_gen<G: Gen>() -> Storage<G> {
        Self::new_with_gen_and_block_capacity(1024)
    }

    pub fn new_with_gen_and_block_capacity<G: Gen>(capacity: usize) -> Storage<G> {
        Storage {
            data: Default::default(),
            capacity,
        }
    }
}

impl<G: Gen> Storage<G> {
    pub fn place<T: 'static>(&mut self, data: T) -> Tid<T, G> {
        let (index, gen) = unsafe {
            self.data
                .entry(TypeId::of::<T>())
                .or_insert(Bucket::new::<T>())
                .place_unchecked(self.capacity, data)
        };

        Tid::new(index, gen)
    }

    pub fn place_id<T: 'static>(&mut self, data: T) -> Id<G> {
        let type_id = TypeId::of::<T>();
        let (index, gen) = unsafe {
            self.data
                .entry(type_id)
                .or_insert(Bucket::new::<T>())
                .place_unchecked(self.capacity, data)
        };

        Id::new(index, gen, type_id)
    }

    pub fn remove<T: 'static>(&mut self, id: impl AsTid<T, G>) -> Option<T> {
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

    pub fn get<T: 'static>(&self, id: impl AsTid<T, G>) -> Option<&T> {
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

    pub fn get_mut<T: 'static>(&mut self, id: impl AsTid<T, G>) -> Option<&mut T> {
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

    pub fn contains<T: 'static>(&self, id: impl AsTid<T, G>) -> bool {
        match id.as_tid() {
            Some(tid) => match self.data.get(&TypeId::of::<T>()) {
                Some(bucket) => bucket.contains(self.capacity, tid.index),
                None => false,
            },
            None => false,
        }
    }

    pub fn contains_id(&self, id: &Id<G>) -> bool {
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

    // remove all placed data, reset all gens, reset all banned cells
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

impl<G: Gen> Drop for Storage<G> {
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

        let mut storage = Storage::new_with_gen::<crate::gen::NoGen>();
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
