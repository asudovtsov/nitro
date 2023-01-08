use std::mem;use std::alloc::Layout;
use std::alloc;

use crate::block::Chunk;

pub(crate) struct Index {
    free_chunks: Vec<Chunk>
}

impl Index {
    pub fn len(&self) -> usize {
        self.free_chunks.len()
    }

    pub fn chunk_at(&self, index: usize) -> &Chunk {
        &self.free_chunks[index]
    }

    pub fn remove_chunk_at(&mut self, index: usize) -> Chunk {
        self.free_chunks.remove(index)
    }

    // finding a chunk with sufficient capacity using a lower bound algorithm
    pub fn lower_bound_free_capacity(&self, capacity: usize) -> Option<usize> {
        let mut left = 0;
        let mut len = self.free_chunks.len();
        let mut index;
        let mut mid;

        while len > 0 {
            index = left;
            mid = len / 2;

            index += mid;
            if self.free_chunks[index].capacity() < capacity {
                if index == self.free_chunks.len() - 1 {
                    return None;
                }

                left = index + 1;
                len -= mid + 1;
                continue;
            }
            len = mid;
        }
        Some(left)
    }

    pub fn chunk_for_place<T>(&self) -> Option<usize> {
        let size = mem::size_of::<T>();
        if let Some(bound) = self.lower_bound_free_capacity(size) {
            for i in bound..self.free_chunks.len() {
                if self.free_chunks[i].is_can_place::<T>() {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn try_merge_chunks(exists_chunk: &mut Chunk, new_chunk: &Chunk) -> bool {
        if exists_chunk.is_next(new_chunk) {
            exists_chunk.add_capacity(new_chunk.capacity());
            return true;
        }
        if new_chunk.is_next(exists_chunk) {
            exists_chunk.copy_start(new_chunk);
            exists_chunk.add_capacity(new_chunk.capacity());
            return true;
        }
        false
    }

    // binary search for new chunk position
    pub fn insert_free_chunk(&mut self, mut chunk: Chunk) -> usize {
        if self.free_chunks.is_empty() {
            self.free_chunks.push(chunk);
            return 0;
        }

        let mut index = self.free_chunks.len() / 2;
        loop {
            let current = &mut self.free_chunks[index];
            let step = index / 2;
            if current.capacity() == chunk.capacity() || step == 0 {
                self.free_chunks.insert(index + 1, chunk);
                break index + 1;
            }

            if current.capacity() < chunk.capacity() {
                index += index / 2;
            } else {
                index -= index / 2;
            }
        }
    }

    // binary search for new chunk position
    // and attempt to merge with existing chunk on each iteration
    pub fn merge_insert_free_chunk(&mut self, mut chunk: Chunk) -> usize {
        if self.free_chunks.is_empty() {
            self.free_chunks.push(chunk);
            return 0;
        }

        let mut index = self.free_chunks.len() / 2;
        loop {
            if Self::try_merge_chunks(&mut self.free_chunks[index], &chunk) {
                chunk = self.free_chunks.remove(index); //#TODO replace with swap
                index = self.free_chunks.len() / 2;
            }

            let current = &mut self.free_chunks[index];
            let step = index / 2;
            if current.capacity() == chunk.capacity() || step == 0 {
                self.free_chunks.insert(index + 1, chunk);
                break index + 1;
            }

            if current.capacity() < chunk.capacity() {
                index += index / 2;
            } else {
                index -= index / 2;
            }
        }
    }

    pub fn alloc_index() -> *mut Index {
        let layout = Layout::new::<Index>();
        let Ok(layout) = layout.align_to(mem::align_of::<Index>()) else {
            panic!("align error")
        };

        unsafe {
            let index: *mut Index = alloc::alloc(layout).cast();
            assert_eq!(index.align_offset(mem::align_of::<Index>()), 0);
            index.write(Index { free_chunks: vec![] });
            index
        }
    }

    pub fn drop_index(index: *mut Index) {
        let layout = Layout::new::<Index>();
        let Ok(layout) = layout.align_to(mem::align_of::<Index>()) else {
            todo!()
        };

        unsafe {
            index.drop_in_place();
            alloc::dealloc(index.cast(), layout);
        }
    }
}