pub trait Gen: Copy + Default {
    fn next(self) -> Self;
    fn is_over(&self) -> bool;
    fn max(&self) -> u128;
    fn current(&self) -> u128;
}

#[derive(Copy, Clone, Default, Debug)]
pub struct NoGen;

impl Gen for NoGen {
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

#[derive(Copy, Clone, Default, Debug)]
pub struct Unique32(u32);

impl Gen for Unique32 {
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

#[derive(Copy, Clone, Default, Debug)]
pub struct Unique64(u64);

impl Gen for Unique64 {
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

#[derive(Copy, Clone, Default, Debug)]
pub struct Unique128(u128);

impl Gen for Unique128 {
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

#[derive(Copy, Clone, Default, Debug)]
pub struct RepeatIn32(u32);

impl Gen for RepeatIn32 {
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

#[derive(Copy, Clone, Default, Debug)]
pub struct RepeatIn64(u64);

impl Gen for RepeatIn64 {
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

#[derive(Copy, Clone, Default, Debug)]
pub struct RepeatIn128(u128);

impl Gen for RepeatIn128 {
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
