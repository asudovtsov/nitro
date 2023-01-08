use crate::block::Block;

pub struct ArenaBox<T> {
    block: *mut Block,
    data: *mut T,
}

impl<T> ArenaBox<T> {
    pub(crate) fn new(block: *mut Block, data: *mut T) -> Self {
        assert!(!block.is_null());
        assert!(!data.is_null());
        ArenaBox {
            block,
            data,
        }
    }
}

impl<T> Drop for ArenaBox<T> {
    fn drop(&mut self) {

    }
}