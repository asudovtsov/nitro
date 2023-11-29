use core::hash::Hash;
use std::fmt::Debug;

pub trait UniqueTag: Copy + Clone + Eq + PartialEq + Default + Hash + Debug {
    fn next(self) -> Self;
    fn is_over(&self) -> bool;
    fn max(&self) -> u128;
    fn current(&self) -> u128;
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct NoTag;

impl UniqueTag for NoTag {
    fn next(self) -> Self {
        self
    }
    fn is_over(&self) -> bool {
        false
    }
    fn max(&self) -> u128 {
        0
    }
    fn current(&self) -> u128 {
        0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct Unique32(u32);

impl UniqueTag for Unique32 {
    fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
    fn is_over(&self) -> bool {
        self.0 == u32::MAX
    }
    fn max(&self) -> u128 {
        u32::MAX as _
    }
    fn current(&self) -> u128 {
        self.0 as _
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct Unique64(u64);

impl UniqueTag for Unique64 {
    fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
    fn is_over(&self) -> bool {
        self.0 == u64::MAX
    }
    fn max(&self) -> u128 {
        u64::MAX as _
    }
    fn current(&self) -> u128 {
        self.0 as _
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct Unique128(u128);

impl UniqueTag for Unique128 {
    fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
    fn is_over(&self) -> bool {
        self.0 == u128::MAX
    }
    fn max(&self) -> u128 {
        u128::MAX
    }
    fn current(&self) -> u128 {
        self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct RepeatIn32(u32);

impl UniqueTag for RepeatIn32 {
    fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    fn is_over(&self) -> bool {
        false
    }
    fn max(&self) -> u128 {
        u32::MAX as _
    }
    fn current(&self) -> u128 {
        self.0 as _
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct RepeatIn64(u64);

impl UniqueTag for RepeatIn64 {
    fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    fn is_over(&self) -> bool {
        false
    }
    fn max(&self) -> u128 {
        u64::MAX as _
    }
    fn current(&self) -> u128 {
        self.0 as _
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct RepeatIn128(u128);

impl UniqueTag for RepeatIn128 {
    fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    fn is_over(&self) -> bool {
        false
    }
    fn max(&self) -> u128 {
        u128::MAX
    }
    fn current(&self) -> u128 {
        self.0
    }
}
