pub trait Cycle: Copy + Default {
    fn next(self) -> Self;
    fn is_over(&self) -> bool;
    fn max(&self) -> u128;
    fn current(&self) -> u128;
}

#[derive(Copy, Clone, Default, Debug)]
pub struct NoCycle;

#[derive(Copy, Clone, Default, Debug)]
pub struct Blocking32(u32);

#[derive(Copy, Clone, Default, Debug)]
pub struct Blocking64(u64);

#[derive(Copy, Clone, Default, Debug)]
pub struct Blocking128(u128);

#[derive(Copy, Clone, Default, Debug)]
pub struct NonBlocking32(u32);

#[derive(Copy, Clone, Default, Debug)]
pub struct NonBlocking64(u64);

#[derive(Copy, Clone, Default, Debug)]
pub struct NonBlocking128(u128);

impl Cycle for NoCycle {
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

impl Cycle for Blocking32 {
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

impl Cycle for Blocking64 {
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

impl Cycle for Blocking128 {
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

impl Cycle for NonBlocking32 {
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

impl Cycle for NonBlocking64 {
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

impl Cycle for NonBlocking128 {
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
