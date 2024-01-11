use crate::{
    bucket::Bucket,
    id::Id,
    params::{Size, UniqueTag},
    storage::Storage,
};
use core::any::TypeId;

pub struct IdAccess<'a, U: UniqueTag, S: Size> {
    storage: &'a Storage<U, S>,
}

impl<'a, U: UniqueTag, S: Size> IdAccess<'a, U, S> {
    pub(crate) fn new(storage: &'a Storage<U, S>) -> Self {
        Self { storage }
    }

    pub fn get<T: 'static>(&self, id: &Id<U, S>) -> Option<&T> {
        if TypeId::of::<T>() != id.type_id() {
            return None;
        }
        match self.storage.data.get(&id.type_id()) {
            Some(bucket) => unsafe { bucket.get_unchecked(id.index()) },
            _ => None,
        }
    }

    pub fn contains(&self, id: &Id<U, S>) -> bool {
        match self.storage.data.get(&id.type_id()) {
            Some(bucket) => unsafe { bucket.contains(id.tag(), id.index()) },
            None => false,
        }
    }
    //#TODO placer?
}

pub struct IdAccessMut<'a, U: UniqueTag, S: Size> {
    storage: &'a mut Storage<U, S>,
}

impl<'a, U: UniqueTag, S: Size> IdAccessMut<'a, U, S> {
    pub(crate) fn new(storage: &'a mut Storage<U, S>) -> Self {
        Self { storage }
    }

    pub fn place<T: 'static>(&mut self, data: T) -> Id<U, S> {
        let type_id = TypeId::of::<T>();
        match unsafe {
            self.storage
                .data
                .entry(type_id)
                .or_insert(Bucket::new::<T>())
                .place_unchecked(data)
        } {
            Ok((index, tag)) => Id::new(index, tag, type_id),
            Err(_) => panic!(),
        }
    }

    pub fn remove<T: 'static>(&mut self, id: &Id<U, S>) -> Option<T> {
        if TypeId::of::<T>() != id.type_id() {
            return None;
        }
        match self.storage.data.get_mut(&id.type_id()) {
            Some(bucket) => unsafe { bucket.remove_unchecked(id.index()) },
            _ => None,
        }
    }

    pub fn get<T: 'static>(&self, id: &Id<U, S>) -> Option<&T> {
        if TypeId::of::<T>() != id.type_id() {
            return None;
        }
        match self.storage.data.get(&id.type_id()) {
            Some(bucket) => unsafe { bucket.get_unchecked(id.index()) },
            _ => None,
        }
    }

    pub fn get_mut<T: 'static>(&mut self, id: &Id<U, S>) -> Option<&mut T> {
        if TypeId::of::<T>() != id.type_id() {
            return None;
        }
        match self.storage.data.get_mut(&id.type_id()) {
            Some(bucket) => unsafe { bucket.get_mut_unchecked(id.index()) },
            _ => None,
        }
    }

    pub fn contains(&self, id: &Id<U, S>) -> bool {
        match self.storage.data.get(&id.type_id()) {
            Some(bucket) => unsafe { bucket.contains(id.tag(), id.index()) },
            None => false,
        }
    }
}
