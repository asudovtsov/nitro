use crate::params::{Size, Unique32, UniqueTag};
use core::any::TypeId;

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Tid<T, U: UniqueTag = Unique32, S: Size = u32> {
    index: S,
    tag: U,
    phantom: std::marker::PhantomData<T>,
}

impl<T, U: UniqueTag, S: Size> Tid<T, U, S> {
    pub(crate) fn new(inbucket_index: S, tag: U) -> Self {
        Self {
            index: inbucket_index,
            tag,
            phantom: Default::default(),
        }
    }

    pub(crate) fn index(&self) -> S {
        self.index
    }

    pub(crate) fn tag(&self) -> U {
        self.tag
    }
}

impl<T, U: UniqueTag, S: Size> Clone for Tid<T, U, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, U: UniqueTag, S: Size> Copy for Tid<T, U, S> {}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Id<U: UniqueTag = Unique32, S: Size = u32> {
    index: S,
    tag: U,
    type_id: TypeId,
}

impl<U: UniqueTag, S: Size> Id<U, S> {
    pub(crate) fn new(inbucket_index: S, tag: U, type_id: TypeId) -> Self {
        Self {
            index: inbucket_index,
            tag,
            type_id,
        }
    }

    pub(crate) fn index(&self) -> S {
        self.index
    }

    pub(crate) fn tag(&self) -> U {
        self.tag
    }

    pub(crate) fn type_id(&self) -> TypeId {
        self.type_id
    }
}

impl<T: 'static, U: UniqueTag> From<Tid<T, U>> for Id<U> {
    fn from(value: Tid<T, U>) -> Self {
        Self {
            index: value.index,
            tag: value.tag,
            type_id: TypeId::of::<T>(),
        }
    }
}
