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

pub trait Palette<T>
where
    T: Eq + Hash + Clone,
    Self: Index<usize, Output = Entry<T>> + IndexMut<usize> + Default,
{
    fn entries(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn index_size(&self) -> u32;
    fn mark_unused(&mut self, index: usize);
    fn get(&self, index: usize) -> Option<&Entry<T>>;
    fn get_mut(&mut self, index: usize) -> Option<&mut Entry<T>>;
    fn entry_for(&self, value: &T) -> Option<(&Entry<T>, usize)>;
    fn entry_for_mut(&mut self, value: &T) -> Option<(&mut Entry<T>, usize)>;
    fn insert_new(&mut self, entry: Entry<T>) -> (usize, Option<u32>);
    fn optimize(&mut self) -> Option<HashMap<usize, usize>>;
}
