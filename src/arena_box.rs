use std::mem;

// use crate::block::{Chunk, Block};

// pub struct ArenaBox<T> {
//     block: *mut Block,
//     data: *mut T,
// }

// impl<T> ArenaBox<T> {
//     pub(crate) fn new(block: *mut Block, data: *mut T) -> Self {
//         assert!(!block.is_null());
//         assert!(!data.is_null());

//         unsafe{&mut (*block)}.increment_counter();

//         ArenaBox {
//             block,
//             data,
//         }
//     }
// }

// impl<T: std::fmt::Debug> ArenaBox<T> {
//     //#TODO remove
//     pub fn print(&self) {
//         println!("{:?}", unsafe{&(*self.data)});
//     }
// }

// impl<T> Drop for ArenaBox<T> {
//     fn drop(&mut self) {
//         let block;
//         unsafe {
//             block = &mut *self.block;
//             self.data.drop_in_place();
//         }

//         if block.counter() == 1 && !block.has_index() {
//             Block::drop_block(block);
//             return;
//         }

//         block.decrement_counter();

//         if block.has_index() {
//             let chunk = Chunk::new(self.block, self.data.cast(), mem::size_of::<T>());
//             block.merge_insert_free_chunk_to_index(chunk);
//         }
//     }
// }