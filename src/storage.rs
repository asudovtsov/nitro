use crate::{
    bucket::Bucket,
    id::Tid,
    id_access::{IdAccess, IdAccessMut},
    params::{Size, Unique32, UniqueTag},
};
use core::any::TypeId;
use std::collections::HashMap;

pub struct Storage<U: UniqueTag = Unique32, S: Size = u32> {
    pub(crate) data: HashMap<TypeId, Bucket<U, S>>,
}

impl Storage<Unique32, u32> {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl Storage {
    pub fn new_with_tag_and_size<U: UniqueTag, S: Size>() -> Storage<U, S> {
        Storage {
            data: Default::default(),
        }
    }
}

impl<U: UniqueTag, S: Size> Storage<U, S> {
    pub fn place<T: 'static>(&mut self, data: T) -> Tid<T, U, S> {
        match unsafe {
            self.data
                .entry(TypeId::of::<T>())
                .or_insert(Bucket::new::<T>())
                .place_unchecked(data)
        } {
            Ok((index, tag)) => Tid::new(index, tag),
            Err(_) => panic!(),
        }
    }

    pub fn remove<T: 'static>(&mut self, id: &Tid<T, U, S>) -> Option<T> {
        match self.data.get_mut(&TypeId::of::<T>()) {
            Some(bucket) => unsafe { bucket.remove_unchecked(id.index()) },
            _ => None,
        }
    }

    pub fn get<T: 'static>(&self, id: &Tid<T, U, S>) -> Option<&T> {
        match self.data.get(&TypeId::of::<T>()) {
            Some(bucket) => unsafe { bucket.get_unchecked(id.index()) },
            _ => None,
        }
    }

    pub fn get_mut<T: 'static>(&mut self, id: &Tid<T, U, S>) -> Option<&mut T> {
        match self.data.get_mut(&TypeId::of::<T>()) {
            Some(bucket) => unsafe { bucket.get_mut_unchecked(id.index()) },
            _ => None,
        }
    }

    pub fn contains<T: 'static>(&self, id: &Tid<T, U, S>) -> bool {
        match self.data.get(&TypeId::of::<T>()) {
            Some(bucket) => unsafe { bucket.contains(id.tag(), id.index()) },
            None => false,
        }
    }

    pub fn shrink_to_fit(&mut self) {
        for (_, bucket) in self.data.iter_mut() {
            unsafe {
                bucket.shrink_to_fit();
            }
        }
        self.data.shrink_to_fit();
    }

    // remove all placed data
    pub fn clear(&mut self) {
        for (_, bucket) in self.data.iter_mut() {
            unsafe {
                Bucket::clear(bucket);
            }
        }
    }

    pub fn clear_bucket<T>(&mut self) {
        todo!()
    }

    pub fn clear_exact(&mut self, target: &[TypeId]) {
        todo!()
    }

    pub fn clear_exclude(&mut self, target: &[TypeId]) {
        todo!()
    }

    // remove all placed data, reset all tags, reset all locked cells
    pub fn reset(&mut self) {
        for (_, bucket) in self.data.iter_mut() {
            unsafe {
                Bucket::reset(bucket);
            }
        }
    }

    pub fn reset_bucket<T>(&mut self) {
        todo!()
    }

    pub fn reset_exact(&mut self, target: &[TypeId]) {
        todo!()
    }

    pub fn reset_exclude(&mut self, target: &[TypeId]) {
        todo!()
    }

    pub fn id(&self) -> IdAccess<'_, U, S> {
        IdAccess::new(self)
    }

    pub fn id_mut(&mut self) -> IdAccessMut<'_, U, S> {
        IdAccessMut::new(self)
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl<U: UniqueTag, S: Size> Drop for Storage<U, S> {
    fn drop(&mut self) {
        for (_, bucket) in self.data.iter_mut() {
            unsafe { Bucket::drop(bucket) }
        }
    }
}
