use hashbrown::HashMap;

mod aligned;

pub use aligned::AlignedIndexBuffer;

pub trait IndexBuffer: Default {
    fn truncate_cleared(&mut self, len: usize);
    fn indices(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn set_index_size(&mut self, new_size: usize, new_mapping: Option<HashMap<usize, usize>>);
    fn set(&mut self, offset: usize, value: usize) -> usize;
    fn get(&self, offset: usize) -> usize;
    fn push(&mut self, index: usize);
    fn pop(&mut self) -> Option<usize>;

    fn iter(&self) -> Iter<'_, Self> {
        Iter {
            buffer: self,
            offset: 0,
        }
    }

    fn offset_iter(&self) -> OffsetIter<'_, Self> {
        OffsetIter {
            buffer: self,
            offset: 0,
        }
    }
}

pub struct OffsetIter<'a, B: IndexBuffer> {
    buffer: &'a B,
    offset: usize,
}

impl<B: IndexBuffer> Iterator for OffsetIter<'_, B> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        (self.offset < self.buffer.indices()).then(|| {
            let idx = self.buffer.get(self.offset);
            let offset = self.offset;
            self.offset += 1;
            (offset, idx)
        })
    }
}

pub struct Iter<'a, B: IndexBuffer> {
    buffer: &'a B,
    offset: usize,
}

impl<B: IndexBuffer> Iterator for Iter<'_, B> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        (self.offset < self.buffer.indices()).then(|| {
            let index = self.buffer.get(self.offset);
            self.offset += 1;
            index
        })
    }
}
