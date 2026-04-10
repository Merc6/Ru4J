use std::{hash::Hash, marker::PhantomData};

use crate::{
    index::{self, AlignedIndexBuffer, IndexBuffer},
    palette::{Entry, Palette, hybrid::HybridPalette},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PVec<T, P = HybridPalette<T>, B = AlignedIndexBuffer>
where
    T: Eq + Hash + Clone,
    P: Palette<T>,
    B: IndexBuffer,
{
    palette: P,
    buffer: B,
    _phantom: PhantomData<fn(T)>,
}

impl<T, P, B> PVec<T, P, B>
where
    T: Eq + Hash + Clone,
    P: Palette<T>,
    B: IndexBuffer,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filled(value: T, len: usize) -> Self {
        let mut palette = P::default();

        let (index, index_size) = palette.insert_new(Entry { value, count: len });
        debug_assert_eq!(index, 0);

        let mut buffer = B::default();

        if let Some(index_size) = index_size {
            buffer.set_index_size(index_size as usize, None);
        }

        buffer.truncate_cleared(len);

        Self {
            palette,
            buffer,
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.indices()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn unique_values(&self) -> usize {
        self.palette.entries()
    }

    pub fn push(&mut self, value: T) {
        match self.palette.entry_for_mut(&value) {
            None => {
                let (idx, new_idx_size) = self.palette.insert_new(Entry { value, count: 1 });

                if let Some(new_idx_size) = new_idx_size {
                    self.buffer.set_index_size(new_idx_size as usize, None);
                }

                self.buffer.push(idx);
            }
            Some((entry, idx)) => {
                entry.count += 1;
                self.buffer.push(idx);
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let idx = self.buffer.pop()?;
        let entry = self.palette.get_mut(idx)?;
        let value = entry.value.clone();

        entry.count -= 1;

        if entry.count == 0 {
            self.palette.mark_unused(idx);
        }

        Some(value)
    }

    pub fn get(&self, offset: usize) -> Option<&T> {
        (offset < self.buffer.indices()).then(|| &self.palette[self.buffer.get(offset)].value)
    }

    pub fn contains(&self, offset: usize) -> bool {
        self.get(offset).is_some()
    }

    pub fn optimize(&mut self) {
        let mapping = self.palette.optimize();
        let new_index_size = self.palette.index_size();
        self.buffer.set_index_size(new_index_size as usize, mapping);
    }

    pub fn iter(&self) -> Iter<'_, T, P, B> {
        self.into_iter()
    }

    pub fn index_iter(&self) -> IndexIter<'_, T, P, B> {
        IndexIter {
            palette: &self.palette,
            buffer: self.buffer.offset_iter(),
            _phantom: PhantomData,
        }
    }
}

impl<T, P, B> Default for PVec<T, P, B>
where
    T: Eq + Hash + Clone,
    P: Palette<T>,
    B: IndexBuffer,
{
    fn default() -> Self {
        Self {
            palette: P::default(),
            buffer: B::default(),
            _phantom: PhantomData,
        }
    }
}

impl<V, P, B> FromIterator<V> for PVec<V, P, B>
where
    V: Eq + Hash + Clone,
    P: Palette<V>,
    B: IndexBuffer,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut ret = Self::default();

        for value in iter {
            ret.push(value);
        }

        ret
    }
}

pub struct Iter<'a, T, P, B>
where
    T: Eq + Hash + Clone + 'a,
    P: Palette<T>,
    B: IndexBuffer,
{
    palette: &'a P,
    buffer: index::Iter<'a, B>,
    _phantom: PhantomData<fn() -> &'a T>,
}

impl<'a, T, P, B> Iterator for Iter<'a, T, P, B>
where
    T: Eq + Hash + Clone,
    P: Palette<T>,
    B: IndexBuffer,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.buffer.next()?;
        let entry = &self.palette[idx];
        Some(&entry.value)
    }
}

impl<'a, T, P, B> IntoIterator for &'a PVec<T, P, B>
where
    T: Eq + Hash + Clone + 'a,
    P: Palette<T>,
    B: IndexBuffer,
{
    type IntoIter = Iter<'a, T, P, B>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            palette: &self.palette,
            buffer: self.buffer.iter(),
            _phantom: PhantomData,
        }
    }
}

pub struct IndexIter<'a, T, P, B>
where
    T: Eq + Hash + Clone + 'a,
    P: Palette<T>,
    B: IndexBuffer,
{
    palette: &'a P,
    buffer: index::OffsetIter<'a, B>,
    _phantom: PhantomData<fn() -> &'a T>,
}

impl<'a, T, P, B> Iterator for IndexIter<'a, T, P, B>
where
    T: Eq + Hash + Clone,
    P: Palette<T>,
    B: IndexBuffer,
{
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, idx) = self.buffer.next()?;
        let entry = &self.palette[idx];
        Some((offset, &entry.value))
    }
}
