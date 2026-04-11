use std::{hash::Hash, marker::PhantomData};

use crate::{
    index::{self, AlignedIndexBuffer, IndexBuffer},
    palette::{Entry, Palette, hybrid::HybridPalette},
};

/// A data-structure similar to a [`Vec`], but utilizes palette compression in
/// how it stores items.
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
    /// Constructs a new [`PVec`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a new [`PVec`], containing `value` repeated `len` times.
    #[must_use]
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

    /// Returns the number of items stored in this [`PVec`].
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the [`PVec`] contains no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the [`PVec's`](PVec) unique entries.
    #[must_use]
    pub fn unique_values(&self) -> usize {
        self.palette.entries()
    }

    /// Appends an item to the end of the [`PVec`].
    pub fn push(&mut self, value: T) {
        match self.palette.find_mut(&value) {
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

    /// Removes and returns the item at the end of the [`PVec`], unless the
    /// `PVec` is empty, then this returns [`None`].
    pub fn pop(&mut self) -> Option<T> {
        let idx = self.buffer.pop()?;
        let entry = self.palette.get_mut(idx)?;
        let value = entry.value.clone();

        entry.count -= 1;

        if entry.count == 0 {
            self.palette.free(idx);
        }

        Some(value)
    }

    /// Returns a reference to the value at `offset`, otherwise returns [`None`]
    #[must_use]
    pub fn get(&self, offset: usize) -> Option<&T> {
        self.buffer.get(offset).map(|idx| &self.palette[idx].value)
    }

    /// Returns `true` if `offset` is valid within [`PVec`].
    #[must_use]
    pub fn contains(&self, offset: usize) -> bool {
        self.buffer.get(offset).is_some()
    }

    /// Optimizes the stored values, could potentially decrease memory
    /// allocated.
    pub fn optimize(&mut self) {
        let mapping = self.palette.optimize();
        let new_index_size = self.palette.index_width();
        self.buffer.set_index_size(new_index_size as usize, mapping);
    }

    /// Returns an [`Iterator`] over the values stored in the [`PVec`].
    pub fn iter(&self) -> Iter<'_, T, P, B> {
        self.into_iter()
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

#[must_use]
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
