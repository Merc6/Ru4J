//! `Palettes` are how the data is stored in its compressed state.

pub mod hybrid;

use std::{
    hash::Hash,
    ops::{Index, IndexMut},
};

use hashbrown::HashMap;

/// A unique-value/count pair to be stored in a `Palette`
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entry<T: Eq + Hash + Clone> {
    /// The unique value being stored.
    pub value: T,

    /// The number of this value being stored.
    pub count: usize,
}

impl<T: Eq + Hash + Clone> Entry<T> {
    /// Constructs a new [`Entry`].
    #[must_use]
    pub const fn new(count: usize, value: T) -> Self {
        Self { value, count }
    }
}

/// A trait for compressing finite and repetitive sets of data `T`.
///
/// This is not to be confused with [collections](crate::collection), palettes
/// exist solely to compress the data, not operate on it. The reason for the
/// separation between palette and collection is because there is no
/// "one-size-fits-all" of palette-compression implementations. It is up to the
/// users to determine the best
/// palette-[`IndexBuffer`](crate::index::IndexBuffer) pair for their use-case.
pub trait Palette<T>
where
    T: Eq + Hash + Clone,
    Self: Index<usize, Output = Entry<T>> + IndexMut<usize> + Default,
{
    /// Returns the number of unique entries stored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lce_palette::palette::{Entry, Palette as _, hybrid::HybridPalette};
    ///
    /// let mut palette = HybridPalette::<u8>::new();
    /// assert_eq!(palette.entries(), 0);
    ///
    /// // insert two ones.
    /// palette.insert_new(Entry::new(2, 1));
    ///
    /// assert_eq!(palette.entries(), 1);
    ///
    /// // insert one two.
    /// palette.insert_new(Entry::new(1, 2));
    /// assert_eq!(palette.entries(), 2);
    /// ```
    fn entries(&self) -> usize;

    /// Returns the minimum number of bits necessary to represent the number of
    /// unique [`Entries`](Entry).
    fn index_width(&self) -> u32;

    /// Frees the entry stored at `index`.
    ///
    /// # Panics
    ///
    /// - Panics if `index` is out of bounds.
    fn free(&mut self, index: usize);

    /// Returns a reference to the [`Entry`] at `index` if index is in this
    /// [`Palette`], otherwise returns [`None`].
    fn get(&self, index: usize) -> Option<&Entry<T>>;

    /// Returns a mutable reference to the [`Entry`] at `index` if index is in
    /// this [`Palette`], otherwise returns [`None`].
    fn get_mut(&mut self, index: usize) -> Option<&mut Entry<T>>;

    /// Returns a reference to the [`Entry`], and the entry's index, for `value`
    /// if it exists, otherwise returns [`None`].
    fn find(&self, value: &T) -> Option<(&Entry<T>, usize)>;

    /// Returns a mutable reference to the [`Entry`], and the entry's index, for
    /// `value` if it exists, otherwise returns [`None`].
    fn find_mut(&mut self, value: &T) -> Option<(&mut Entry<T>, usize)>;

    /// Inserts a **new** [`Entry`] into this [`Palette`], returning its index,
    /// and the new [index-width](Palette::index_width) if it changed.
    fn insert_new(&mut self, entry: Entry<T>) -> (usize, Option<u32>);

    /// Optimizes the stored [`Entries`](Entry) for this [`Palette`], returning
    /// the old->new mappings if they changed.
    ///
    /// This is an expensive function that should rarely be called - but should
    /// be called.
    fn optimize(&mut self) -> Option<HashMap<usize, usize>>;

    /// Returns `true` if the [`Palette`] contains no entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lce_palette::palette::{Entry, Palette as _, hybrid::HybridPalette};
    ///
    /// let mut palette = HybridPalette::<u8>::new();
    /// assert!(palette.is_empty());
    ///
    /// palette.insert_new(Entry::new(1, 1));
    /// assert!(!palette.is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        self.entries() == 0
    }
}
