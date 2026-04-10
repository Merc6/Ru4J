use core::slice;
use std::{
    hash::Hash,
    ops::{Index, IndexMut},
};

use hashbrown::{HashMap, hash_map};

use crate::palette::{Entry, Palette};

#[derive(Clone, Debug)]
pub struct HybridPalette<T, const HEAP_THRESHOLD: usize = { calc_heap_threshold::<T>() }>
where
    T: Eq + Hash + Clone,
{
    index_size: u32,
    real_entries: usize,
    storage: HybridStorage<HEAP_THRESHOLD, T>,
}

impl<const HEAP_THRESHOLD: usize, T> HybridPalette<T, HEAP_THRESHOLD>
where
    T: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn heap_threshold() -> usize {
        return HEAP_THRESHOLD;
    }

    fn convert_to_hashmap(&mut self) {
        let HybridStorage::Array(ref array) = self.storage else {
            unreachable!()
        };

        let mut free_indices = Vec::new();
        let mut index_map = HashMap::with_capacity(HEAP_THRESHOLD);
        let mut value_map = HashMap::with_capacity(HEAP_THRESHOLD);

        for (idx, entry) in array.iter().enumerate() {
            let Some(entry) = entry else {
                free_indices.push(idx);
                continue;
            };

            debug_assert!(entry.count != 0);

            // SAFETY: entries are unique.
            unsafe { value_map.insert_unique_unchecked(entry.value.clone(), idx) };
            unsafe { index_map.insert_unique_unchecked(idx, entry.clone()) };
        }

        debug_assert_eq!(index_map.len(), HEAP_THRESHOLD);

        self.storage = HybridStorage::HashMap {
            free_indices,
            index_map,
            value_map,
        };
    }

    fn convert_to_array(&mut self) -> Option<HashMap<usize, usize>> {
        let HybridStorage::HashMap {
            ref index_map,
            ref value_map,
            ..
        } = self.storage
        else {
            unreachable!()
        };

        debug_assert_eq!(index_map.len(), value_map.len());
        debug_assert!(index_map.len() <= HEAP_THRESHOLD);

        let mut new_mapping = HashMap::with_capacity(HEAP_THRESHOLD);
        let mut array = [const { none() }; HEAP_THRESHOLD];

        let mut index_map = index_map.iter().collect::<Vec<_>>();
        index_map.sort_by(|(ia, a), (ib, b)| b.count.cmp(&a.count).then(ia.cmp(ib)));

        let mut needs_new_mapping = false;
        for (new_idx, (old_idx, entry)) in index_map.iter().enumerate() {
            debug_assert!(entry.count != 0);

            if new_idx != **old_idx {
                needs_new_mapping = true;
            }

            unsafe { new_mapping.insert_unique_unchecked(**old_idx, new_idx) };
            array[new_idx] = Some((*entry).clone());
        }

        self.storage = HybridStorage::Array(array);

        needs_new_mapping.then_some(new_mapping)
    }

    fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }
}

impl<const HEAP_THRESHOLD: usize, T> Palette<T> for HybridPalette<T, HEAP_THRESHOLD>
where
    T: Eq + Hash + Clone,
{
    fn entries(&self) -> usize {
        self.real_entries
    }

    fn is_empty(&self) -> bool {
        self.entries() == 0
    }

    fn index_size(&self) -> u32 {
        self.index_size
    }

    fn mark_unused(&mut self, index: usize) {
        match &mut self.storage {
            HybridStorage::Array(array) => array[index] = None,
            HybridStorage::HashMap {
                free_indices,
                index_map,
                value_map,
            } => {
                let entry = index_map
                    .remove(&index)
                    .expect("index should exist in `index_map`");
                debug_assert_eq!(entry.count, 0);

                free_indices.push(index);
                value_map.remove(&entry.value);
            }
        }
    }

    fn get(&self, index: usize) -> Option<&Entry<T>> {
        match &self.storage {
            HybridStorage::Array(array) => array[index].as_ref(),
            HybridStorage::HashMap { index_map, .. } => index_map.get(&index),
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Entry<T>> {
        match &mut self.storage {
            HybridStorage::Array(array) => array[index].as_mut(),
            HybridStorage::HashMap { index_map, .. } => index_map.get_mut(&index),
        }
    }

    fn entry_for(&self, value: &T) -> Option<(&Entry<T>, usize)> {
        match &self.storage {
            HybridStorage::Array(array) => array.iter().enumerate().find_map(|(idx, entry)| {
                entry
                    .as_ref()
                    .and_then(|entry| (entry.value == *value).then_some((entry, idx)))
            }),
            HybridStorage::HashMap {
                index_map,
                value_map,
                ..
            } => {
                let idx = value_map.get(value)?;
                index_map.get(idx).map(|entry| (entry, *idx))
            }
        }
    }

    fn entry_for_mut(&mut self, value: &T) -> Option<(&mut Entry<T>, usize)> {
        match &mut self.storage {
            HybridStorage::Array(array) => array.iter_mut().enumerate().find_map(|(idx, entry)| {
                entry
                    .as_mut()
                    .and_then(|entry| (entry.value == *value).then_some((entry, idx)))
            }),
            HybridStorage::HashMap {
                index_map,
                value_map,
                ..
            } => {
                let idx = value_map.get(value)?;
                index_map.get_mut(idx).map(|entry| (entry, *idx))
            }
        }
    }

    fn insert_new(&mut self, entry: Entry<T>) -> (usize, Option<u32>) {
        debug_assert!(entry.count != 0);

        match &mut self.storage {
            HybridStorage::Array(array) => {
                if let Some((idx, old_entry)) = array.iter_mut().enumerate().find(|(_, entry)| {
                    entry.is_none() || entry.as_ref().is_some_and(|entry| entry.count == 0)
                }) {
                    old_entry.replace(entry);
                    self.real_entries += 1;

                    let new_index_size = self.real_entries.ilog2() + 1;
                    let mut actual_new_index_size = None;

                    if new_index_size > self.index_size {
                        self.index_size = new_index_size;
                        actual_new_index_size.replace(new_index_size);
                    }

                    return (idx, actual_new_index_size);
                }

                self.convert_to_hashmap();
                self.insert_new(entry)
            }
            HybridStorage::HashMap {
                free_indices,
                index_map,
                value_map,
            } => {
                if let Some(idx) = free_indices.pop() {
                    value_map.insert(entry.value.clone(), idx);
                    index_map.insert(idx, entry);
                    self.real_entries += 1;

                    return (idx, None);
                }

                let idx = index_map.len();

                value_map.insert(entry.value.clone(), idx);
                index_map.insert(idx, entry);
                self.real_entries += 1;

                let new_index_size = self.real_entries.ilog2() + 1;
                let mut actual_new_index_size = None;

                if new_index_size > self.index_size {
                    self.index_size = new_index_size;
                    actual_new_index_size.replace(new_index_size);
                }

                (idx, actual_new_index_size)
            }
        }
    }

    fn optimize(&mut self) -> Option<HashMap<usize, usize>> {
        self.index_size = self.real_entries.checked_ilog2().map_or(0, |x| x + 1);

        match &mut self.storage {
            HybridStorage::Array(array) => {
                let old_mapping = array
                    .iter()
                    .enumerate()
                    .filter_map(|(i, e)| e.as_ref().map(|e| (e.value.clone(), i)))
                    .collect::<HashMap<T, usize>>();

                array.sort_by(|a, b| match (a, b) {
                    (Some(a), Some(b)) => b.count.cmp(&a.count),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });

                let mut new_mapping = HashMap::new();
                let mut needs_new_mapping = false;

                for (new_idx, entry) in array.iter().enumerate() {
                    let Some(entry) = entry else { break };

                    let old_idx = *old_mapping
                        .get(&entry.value)
                        .expect("`old_mapping` should contain current entry");

                    if new_idx != old_idx {
                        needs_new_mapping = true;
                    }

                    new_mapping.insert(old_idx, new_idx);
                }

                needs_new_mapping.then_some(new_mapping)
            }
            HybridStorage::HashMap {
                free_indices,
                index_map,
                value_map,
            } => {
                debug_assert_eq!(index_map.len(), value_map.len());

                if index_map.len() <= HEAP_THRESHOLD {
                    return self.convert_to_array();
                }

                if free_indices.is_empty() {
                    return None;
                }

                let mut new_mapping = HashMap::with_capacity(HEAP_THRESHOLD);
                let mut new_index_map = HashMap::with_capacity(HEAP_THRESHOLD);
                let mut new_value_map = HashMap::with_capacity(HEAP_THRESHOLD);

                let mut entries = index_map.drain().collect::<Vec<_>>();
                entries.sort_by(|(ia, ea), (ib, eb)| eb.count.cmp(&ea.count).then(ia.cmp(ib)));

                for (new_idx, (old_idx, entry)) in entries.into_iter().enumerate() {
                    new_value_map.insert(entry.value.clone(), new_idx);
                    new_index_map.insert(new_idx, entry);
                    new_mapping.insert(old_idx, new_idx);
                }

                free_indices.clear();

                self.storage = HybridStorage::HashMap {
                    free_indices: std::mem::take(free_indices),
                    index_map: new_index_map,
                    value_map: new_value_map,
                };

                Some(new_mapping)
            }
        }
    }
}

impl<const H: usize, T> IndexMut<usize> for HybridPalette<T, H>
where
    T: Eq + Hash + Clone,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<const H: usize, T> Index<usize> for HybridPalette<T, H>
where
    T: Eq + Hash + Clone,
{
    type Output = Entry<T>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<const H: usize, T> Default for HybridPalette<T, H>
where
    T: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            index_size: 0,
            real_entries: 0,
            storage: HybridStorage::default(),
        }
    }
}

impl<'a, const H: usize, T: Eq + Hash + Clone> IntoIterator for &'a HybridPalette<T, H> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a Entry<T>;

    fn into_iter(self) -> Self::IntoIter {
        match &self.storage {
            HybridStorage::Array(array) => Iter::Array(array.iter().filter_map(Option::as_ref)),
            HybridStorage::HashMap { index_map, .. } => Iter::HashMap(index_map.values()),
        }
    }
}

pub enum Iter<'a, T: Eq + Hash + Clone> {
    Array(
        core::iter::FilterMap<
            slice::Iter<'a, Option<Entry<T>>>,
            fn(&Option<Entry<T>>) -> Option<&Entry<T>>,
        >,
    ),
    HashMap(hash_map::Values<'a, usize, Entry<T>>),
}

impl<'a, T: Eq + Hash + Clone> Iterator for Iter<'a, T> {
    type Item = &'a Entry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Array(iter) => iter.next(),
            Self::HashMap(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Array(iter) => iter.size_hint(),
            Self::HashMap(iter) => iter.size_hint(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum HybridStorage<const HEAP_THRESHOLD: usize, T>
where
    T: Eq + Hash + Clone,
{
    Array([Option<Entry<T>>; HEAP_THRESHOLD]),
    HashMap {
        free_indices: Vec<usize>,
        index_map: HashMap<usize, Entry<T>>,
        value_map: HashMap<T, usize>,
    },
}

impl<const HEAP_THRESHOLD: usize, T> Default for HybridStorage<HEAP_THRESHOLD, T>
where
    T: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::Array([const { none() }; HEAP_THRESHOLD])
    }
}

pub const fn calc_heap_threshold<T: Eq + Hash + Clone>() -> usize {
    let hm_sizes = 2 * size_of::<HashMap<usize, usize>>();
    let vec_size = size_of::<Vec<usize>>();

    (hm_sizes + vec_size) / size_of::<Option<Entry<T>>>()
}

const fn none<T>() -> Option<T> {
    None
}
