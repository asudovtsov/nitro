use core::hash::Hash;
use std::fmt::Debug;

pub trait UniqueTag: Copy + Clone + Eq + PartialEq + Default + Hash + Debug {
    fn next(self) -> Self;
    fn last(self) -> usize;
    fn current(self) -> usize;

    fn is_removed(&self) -> bool;
    fn set_removed(&mut self, removed: bool);

    fn is_locked(&self) -> bool;
    fn mark_locked(&mut self);
}

macro_rules! impl_unique {
    ($S:tt, $T:tt) => {
        impl UniqueTag for $S {
            fn next(self) -> Self {
                Self($T::min(self.0 + 1, self.last() as $T))
            }
            fn last(self) -> usize {
                ($T::pow(2, $T::BITS - 1) - 1) as _
            }
            fn current(self) -> usize {
                self.0 as _
            }

            fn is_removed(&self) -> bool {
                self.0 & (1 << ($T::BITS - 1)) != 0
            }
            fn set_removed(&mut self, removed: bool) {
                self.0 = if removed {
                    self.0 | (1 << ($T::BITS - 1))
                } else {
                    self.0 & !(1 << ($T::BITS - 1))
                }
            }

            fn is_locked(&self) -> bool {
                self.0 == $T::pow(2, $T::BITS - 1)
            }
            fn mark_locked(&mut self) {
                self.0 = $T::pow(2, $T::BITS - 1)
            }
        }
    };
}

macro_rules! impl_repeat_in {
    ($S:tt, $T:tt) => {
        impl UniqueTag for $S {
            fn next(self) -> Self {
                Self($T::min(self.0 + 1, self.last() as $T))
            }
            fn last(self) -> usize {
                $T::pow(2, $T::BITS - 1) as _
            }
            fn current(self) -> usize {
                self.0 as _
            }

            fn is_removed(&self) -> bool {
                self.0 & (1 << ($T::BITS - 1)) != 0
            }
            fn set_removed(&mut self, removed: bool) {
                self.0 = if removed {
                    self.0 | (1 << ($T::BITS - 1))
                } else {
                    self.0 & !(1 << ($T::BITS - 1))
                }
            }

            fn is_locked(&self) -> bool {
                false
            }
            fn mark_locked(&mut self) {}
        }
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct Unique32(u32);

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct Unique64(u64);

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct Unique128(u128);

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct RepeatIn32(u32);

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct RepeatIn64(u64);

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct RepeatIn128(u128);

impl_unique!(Unique32, u32);
impl_unique!(Unique64, u64);
impl_unique!(Unique128, u128);

impl_repeat_in!(RepeatIn32, u32);
impl_repeat_in!(RepeatIn64, u64);
impl_repeat_in!(RepeatIn128, u128);

pub trait Size: Copy + Clone + Debug + Default + Eq + PartialEq {
    fn max() -> usize;
    fn from_usize(value: usize) -> Self;
    fn into_usize(self) -> usize;
}

impl Size for u32 {
    fn max() -> usize {
        u32::MAX as _
    }
    fn from_usize(value: usize) -> Self {
        assert!(value <= u32::MAX as usize);
        u32::min(value as _, u32::MAX)
    }
    fn into_usize(self) -> usize {
        self as _
    }
}

impl Size for usize {
    fn max() -> usize {
        usize::MAX
    }
    fn from_usize(value: usize) -> Self {
        value
    }
    fn into_usize(self) -> usize {
        self as _
    }
}
