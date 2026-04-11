//! Storage implementations for indices into palette-compressed data-structures.

use std::iter::FusedIterator;

use hashbrown::HashMap;

mod aligned;

pub use aligned::AlignedIndexBuffer;

/// A trait for dealing with [`Palette`](crate::palette::Palette)
/// [`entries`](crate::palette::Entry) - specifically, how to index them.
///
/// This is kept separate from palettes because palettes aren't a collection,
/// they are a data-compression method; likewise, index-buffers should serve the
/// collection.
pub trait IndexBuffer: Default {
    /// Truncates down to `len` and zeroes-out the buffer.
    fn truncate_cleared(&mut self, len: usize);

    /// The number of indices stored.
    #[must_use]
    fn len(&self) -> usize;

    /// Sets the width of the index to a new size.
    ///
    /// This is a method that exists due to the nature of palette-compression.
    /// For more information about the concept of palette-compression please
    /// see [crate-level documentation](crate).
    fn set_index_size(&mut self, new_size: usize, new_mapping: Option<HashMap<usize, usize>>);

    /// Sets the index at `offset` to `value`; returns the old index at
    /// [`offset`].
    fn set(&mut self, offset: usize, value: usize) -> usize;

    /// Gets the index at `offset`; return [`None`] if there is no index.
    #[must_use]
    fn get(&self, offset: usize) -> Option<usize>;

    /// Gets the index at `offset`.
    ///
    /// # Safety
    ///
    /// - `offset` must be less-than `len()`.
    #[must_use]
    unsafe fn get_unchecked(&self, offset: usize) -> usize;

    /// Appends `index` to the end of the buffer.
    fn push(&mut self, index: usize);

    /// Removes the last index from an `IndexBuffer` and returns it, or [`None`]
    /// if empty.
    fn pop(&mut self) -> Option<usize>;

    /// Returns `true` if the `IndexBuffer` contains no indices.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an [`Iterator`] over the `IndexBuffer`.
    fn iter(&self) -> Iter<'_, Self> {
        Iter {
            buffer: self,
            offset: 0,
        }
    }
}

/// The iterator over an [`IndexBuffer`].
///
/// Yields the stored indices.
#[must_use]
pub struct Iter<'a, B: IndexBuffer> {
    buffer: &'a B,
    offset: usize,
}

impl<B: IndexBuffer> Iterator for Iter<'_, B> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.get(self.offset).inspect(|_| self.offset += 1)
    }
}

impl<B: IndexBuffer> FusedIterator for Iter<'_, B> {}
