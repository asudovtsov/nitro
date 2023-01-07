use std::ptr::NonNull;

use crate::block_list::Block;

pub struct ArenaBox<T> {
    block: NonNull<Block>,
    data: NonNull<T>,
}

impl<T> ArenaBox<T> {
    pub fn new(block: NonNull<Block>, data: NonNull<T>) -> Self {
        ArenaBox {
            block,
            data,
        }
    }
}