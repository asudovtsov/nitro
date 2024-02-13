use crate::{
    bucket::Bucket,
    params::{Size, Unique32, UniqueTag},
    token_bucket::TokenBucket,
    U32Size,
};
use core::any::TypeId;
use std::collections::HashMap;

pub struct Storage<S: Size = U32Size, U: UniqueTag = Unique32> {
    tokens: TokenBucket<S, U>,
    data: Vec<(TypeId, Bucket<S>)>,
    bucket_indexes: HashMap<TypeId, S>,
}

impl Storage<U32Size, Unique32> {
    pub fn new() -> Self {
        Self {
            tokens: TokenBucket::new(),
            data: Vec::new(),
            bucket_indexes: HashMap::new(),
        }
    }
}

impl Storage {
    pub fn new_with_tag_and_size<S: Size, U: UniqueTag>() -> Storage<S, U> {
        Storage {
            tokens: TokenBucket::new(),
            data: Vec::new(),
            bucket_indexes: HashMap::new(),
        }
    }
}

impl<S: Size, U: UniqueTag> Storage<S, U> {
    pub fn place<T: 'static>(&mut self, data: T) -> Id<S, U> {
        let type_id = TypeId::of::<T>();
        let bucket_index = *self
            .bucket_indexes
            .entry(type_id)
            .or_insert(self.data.len().into());

        if bucket_index == self.data.len().into() {
            self.data.push((type_id, Bucket::new::<T>()));
        }

        let bucket = &mut self.data[bucket_index.into()].1;

        match unsafe { bucket.push_unchecked(data) } {
            Ok(inbucket_index) => {
                let (token_index, tag) = self.tokens.create(bucket_index, inbucket_index);
                unsafe {
                    bucket.set_token_index_unchecked::<T>(inbucket_index, token_index);
                }
                Id::new(token_index, tag)
            }
            Err(_) => panic!(),
        }
    }

    pub fn place_at<T: 'static>(bucket_ref: BucketRef<'_, S, U>, data: T) -> Id<S, U> {
        let type_id = TypeId::of::<T>();
        let bucket_index = *bucket_ref.entry.or_insert(bucket_ref.data.len().into());

        if bucket_index == bucket_ref.data.len().into() {
            bucket_ref.data.push((type_id, Bucket::new::<T>()));
        }

        let bucket = &mut bucket_ref.data[bucket_index.into()].1;

        match unsafe { bucket.push_unchecked(data) } {
            Ok(inbucket_index) => {
                let (token_index, tag) = bucket_ref.tokens.create(bucket_index, inbucket_index);
                unsafe {
                    bucket.set_token_index_unchecked::<T>(inbucket_index, token_index);
                }
                Id::new(token_index, tag)
            }
            Err(_) => panic!(),
        }
    }

    pub fn remove<T: 'static>(&mut self, id: &Id<S, U>) -> Option<T> {
        match self.tokens.try_get_token(id.token_index()) {
            Some(token) => {
                if token.tag().is_removed() || token.tag().is_locked() {
                    return None;
                }

                let location = unsafe { *token.location() };
                match self.data.get_mut(location.bucket_index().into()) {
                    Some((type_id, bucket)) => {
                        if TypeId::of::<T>() != *type_id {
                            return None;
                        }

                        self.tokens.mark_removed(id.token_index());

                        let (data, token_index_for_swap) =
                            unsafe { bucket.swap_remove_unchecked::<T>(location.inbucket_index()) };

                        if let Some(token_index) = token_index_for_swap {
                            self.tokens
                                .set_inbucket_index(token_index, location.inbucket_index())
                        }

                        Some(data)
                    }
                    None => None,
                }
            }
            None => None,
        }
    }

    pub fn erase(&mut self, id: &Id<S, U>) {
        if let Some(token) = self.tokens.try_get_token(id.token_index()) {
            if token.tag().is_removed() || token.tag().is_locked() {
                return;
            }

            let location = unsafe { *token.location() };
            if let Some((_, bucket)) = self.data.get_mut(location.bucket_index().into()) {
                self.tokens.mark_removed(id.token_index());

                let token_index_for_swap =
                    unsafe { bucket.swap_erase_unchecked(location.inbucket_index()) };

                if let Some(token_index) = token_index_for_swap {
                    self.tokens
                        .set_inbucket_index(token_index, location.inbucket_index())
                }
            }
        }
    }

    pub fn get<T: 'static>(&self, id: &Id<S, U>) -> &T {
        match self.tokens.try_get_token(id.token_index()) {
            Some(token) => {
                let location = unsafe { *token.location() };
                let (type_id, bucket) = &self.data[location.bucket_index().into()];
                if TypeId::of::<T>() != *type_id {
                    panic!();
                }

                unsafe { bucket.get_unchecked(location.inbucket_index()) }
            }
            None => panic!(),
        }
    }

    pub fn try_get<T: 'static>(&self, id: &Id<S, U>) -> Option<&T> {
        match self.tokens.try_get_token(id.token_index()) {
            Some(token) => {
                let location = unsafe { *token.location() };
                match self.data.get(location.bucket_index().into()) {
                    Some((type_id, bucket)) => {
                        if TypeId::of::<T>() != *type_id {
                            return None;
                        }

                        bucket.try_get(location.inbucket_index())
                    }
                    None => None,
                }
            }
            None => None,
        }
    }

    pub fn get_mut<T: 'static>(&mut self, id: &Id<S, U>) -> &mut T {
        match self.tokens.try_get_token(id.token_index()) {
            Some(token) => {
                let location = unsafe { *token.location() };
                let (type_id, bucket) = &mut self.data[location.bucket_index().into()];
                if TypeId::of::<T>() != *type_id {
                    panic!();
                }

                unsafe { bucket.get_mut_unchecked(location.inbucket_index()) }
            }
            None => panic!(),
        }
    }

    pub fn try_get_mut<T: 'static>(&mut self, id: &Id<S, U>) -> Option<&mut T> {
        match self.tokens.try_get_token(id.token_index()) {
            Some(token) => {
                let location = unsafe { *token.location() };
                match self.data.get_mut(location.bucket_index().into()) {
                    Some((type_id, bucket)) => {
                        if TypeId::of::<T>() != *type_id {
                            return None;
                        }

                        bucket.try_get_mut(location.inbucket_index())
                    }
                    None => None,
                }
            }
            None => None,
        }
    }

    pub fn contains(&self, id: &Id<S, U>) -> bool {
        self.tokens.contains(id.token_index(), id.tag())
    }

    pub fn contains_exact<T: 'static>(&self, id: &Id<S, U>) -> bool {
        match self.tokens.try_get_token(id.token_index()) {
            Some(token) => {
                let usize_bucket_index = unsafe { token.location().bucket_index().into() };
                if usize_bucket_index >= self.data.len() {
                    return false;
                }

                let (type_id, _) = &self.data[usize_bucket_index];
                if TypeId::of::<T>() != *type_id {
                    return false;
                }

                let tag = token.tag();
                id.tag() == token.tag() && !tag.is_removed() && !tag.is_locked()
            }
            None => false,
        }
    }

    pub fn shrink_to_fit(&mut self) {
        self.tokens.shrink_to_fit();
        self.bucket_indexes.shrink_to_fit();
        for (_, bucket) in self.data.iter_mut() {
            unsafe {
                bucket.shrink_to_fit();
            }
        }
        self.data.shrink_to_fit();
    }

    // // remove all placed data
    pub fn clear(&mut self) {
        self.tokens.clear();
        for (_, bucket) in self.data.iter_mut() {
            unsafe {
                Bucket::clear(bucket);
            }
        }
    }

    // remove all placed data, reset all tags, reset all locked cells
    pub fn reset(&mut self) {
        self.tokens.reset_tokens();
        self.bucket_indexes.clear();
        for (_, bucket) in self.data.iter_mut() {
            unsafe {
                Bucket::clear(bucket);
            }
        }
    }

    pub fn bucket_ref<T: 'static>(&mut self) -> BucketRef<'_, S, U> {
        BucketRef {
            tokens: &mut self.tokens,
            data: &mut self.data,
            entry: self.bucket_indexes.entry(TypeId::of::<T>()),
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Size, U: UniqueTag> Drop for Storage<S, U> {
    fn drop(&mut self) {
        for (_, bucket) in self.data.iter_mut() {
            unsafe { Bucket::drop(bucket) }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Id<S: Size, U: UniqueTag> {
    token_index: S,
    tag: U,
}

impl<S: Size, U: UniqueTag> Id<S, U> {
    pub(crate) fn new(token_index: S, tag: U) -> Self {
        Self { tag, token_index }
    }

    pub(crate) fn token_index(&self) -> S {
        self.token_index
    }

    pub(crate) fn tag(&self) -> U {
        self.tag
    }
}

pub struct BucketRef<'a, S: Size, U: UniqueTag> {
    tokens: &'a mut TokenBucket<S, U>,
    data: &'a mut Vec<(TypeId, Bucket<S>)>,
    entry: std::collections::hash_map::Entry<'a, TypeId, S>,
}

impl<'a, S: Size, U: UniqueTag> BucketRef<'a, S, U> {
    pub fn bucket_is_exists(&self) -> bool {
        match self.entry {
            std::collections::hash_map::Entry::Occupied(_) => true,
            std::collections::hash_map::Entry::Vacant(_) => false,
        }
    }
}

mod tests {
    #[test]
    fn place_remove_contains() {
        use super::*;

        type Color = (String, u8, u8, u8);

        let mut storage = Storage::new();
        let red = storage.place::<Color>((String::from("red"), 255, 0, 0));
        let green = storage.place::<Color>((String::from("green"), 0, 255, 0));

        let mut ids = vec![];
        for _ in 0..10_000 {
            ids.push(storage.place::<Color>((String::from("blue"), 255, 0, 0)));
        }

        for id in ids.iter() {
            storage.remove::<Color>(id);
        }

        assert!(storage.contains(&red));
        assert!(storage.contains(&green));

        storage.remove::<Color>(&red);
        assert!(!storage.contains(&red));
        assert!(storage.contains(&green));

        storage.remove::<Color>(&green);
        assert!(!storage.contains(&red));
        assert!(!storage.contains(&green));
    }

    #[test]
    fn place_erase_contains() {
        use super::*;

        type Color = (String, u8, u8, u8);

        let mut storage = Storage::new();
        let red = storage.place::<Color>((String::from("red"), 255, 0, 0));
        let green = storage.place::<Color>((String::from("green"), 0, 255, 0));

        let mut ids = vec![];
        for _ in 0..10_000 {
            ids.push(storage.place::<Color>((String::from("blue"), 255, 0, 0)));
        }

        for id in ids.iter() {
            storage.erase(id);
        }

        assert!(storage.contains(&red));
        assert!(storage.contains(&green));

        storage.erase(&red);
        assert!(!storage.contains(&red));
        assert!(storage.contains(&green));

        storage.erase(&green);
        assert!(!storage.contains(&red));
        assert!(!storage.contains(&green));
    }
}
