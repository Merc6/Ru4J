use std::cmp::Ordering;

use crate::index::IndexBuffer;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AlignedIndexBuffer {
    index_size: usize,
    indices_per_u64: u8,
    mask: u64,
    len: usize,
    storage: Vec<u64>,
}

impl AlignedIndexBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    fn set_index_with_index_size(
        &mut self,
        offset: usize,
        index_size: usize,
        indices_per_u64: usize,
        index: usize,
    ) -> usize {
        debug_assert!(index_size > 0);
        debug_assert_eq!(64 / index_size, indices_per_u64);

        let target_u64 = unsafe { self.storage.get_unchecked_mut(offset / indices_per_u64) };
        let target_offset = (offset % indices_per_u64) * index_size;
        let mask = u64::MAX >> (u64::BITS as usize - index_size);
        let old_index = (*target_u64 >> target_offset) & mask;

        *target_u64 &= !(mask << target_offset);
        *target_u64 |= (index as u64) << target_offset;

        old_index
            .try_into()
            .expect("`old_index` should fit in a `usize`")
    }
}

impl IndexBuffer for AlignedIndexBuffer {
    fn truncate_cleared(&mut self, len: usize) {
        self.len = len;

        if self.index_size == 0 {
            debug_assert!(self.storage.is_empty());
            return;
        }

        let indices_per_u64 = 64 / self.index_size;
        let needed_u64 = len.div_ceil(indices_per_u64);

        self.mask = (1 << self.index_size) - 1;

        self.storage.resize(needed_u64, 0);
        self.storage.clear();

        self.indices_per_u64 = indices_per_u64
            .try_into()
            .expect("Indices should fit within a u8");
    }

    fn indices(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn set_index_size(
        &mut self,
        new_size: usize,
        new_mapping: Option<hashbrown::HashMap<usize, usize>>,
    ) {
        match new_size.cmp(&self.index_size) {
            Ordering::Greater => {
                let new_indices_per_u64 = 64 / new_size;
                let needed_u64 = self.len.div_ceil(new_indices_per_u64);

                self.storage.resize(needed_u64, 0);

                match new_mapping {
                    None => (0..self.len).rev().for_each(|idx| {
                        self.set_index_with_index_size(
                            idx,
                            new_size,
                            new_indices_per_u64,
                            self.get(idx),
                        );
                    }),
                    Some(ref mapping) => (0..self.len).rev().for_each(|idx| {
                        self.set_index_with_index_size(
                            idx,
                            new_size,
                            new_indices_per_u64,
                            *mapping
                                .get(&self.get(idx))
                                .expect("mapping should contain old index"),
                        );
                    }),
                }

                self.mask = (1 << new_size) - 1;
                self.indices_per_u64 = new_indices_per_u64
                    .try_into()
                    .expect("new_indices_per_u64 should fit in a u8");
            }
            Ordering::Less => {
                if new_size == 0 {
                    if let Some(ref new_mapping) = new_mapping {
                        debug_assert!(new_mapping.len() == 1);
                        debug_assert!(new_mapping.values().any(|&x| x == 0));
                    }

                    self.index_size = 0;
                    self.storage.clear();

                    return;
                }

                let new_indices_per_u64 = 64 / new_size;

                match new_mapping {
                    None => (0..self.len).for_each(|idx| {
                        self.set_index_with_index_size(
                            idx,
                            new_size,
                            new_indices_per_u64,
                            self.get(idx),
                        );
                    }),
                    Some(ref mapping) => (0..self.len).for_each(|idx| {
                        self.set_index_with_index_size(
                            idx,
                            new_size,
                            new_indices_per_u64,
                            *mapping
                                .get(&self.get(idx))
                                .expect("mapping should contain old index"),
                        );
                    }),
                }

                let needed_u64 = self.len.div_ceil(new_indices_per_u64);

                self.mask = (1 << new_size) - 1;
                self.storage.truncate(needed_u64);
                self.indices_per_u64 = new_indices_per_u64
                    .try_into()
                    .expect("new indices should fit in a u8");
            }
            Ordering::Equal if let Some(ref mapping) = new_mapping => {
                (0..self.len).for_each(|idx| {
                    self.set(
                        idx,
                        *mapping
                            .get(&self.get(idx))
                            .expect("mapping should contain old index"),
                    );
                });
            }
            Ordering::Equal => {}
        }
    }

    fn push(&mut self, index: usize) {
        if self.index_size == 0 {
            self.len += 1;
            return;
        }

        let indices_per_u64: usize = self.indices_per_u64.into();

        if self.len.is_multiple_of(indices_per_u64) {
            self.storage.push(index as u64);
            self.len += 1;
            return;
        }

        self.len += 1;
        self.set(self.len - 1, index);
    }

    fn pop(&mut self) -> Option<usize> {
        if self.len == 0 {
            return None;
        }

        if self.index_size == 0 {
            self.len -= 1;
            return Some(0);
        }

        let indices_per_u64: usize = self.indices_per_u64.into();
        let index = self.get(self.len - 1);

        self.len -= 1;

        if self.len.is_multiple_of(indices_per_u64) {
            self.storage.pop();
        }

        Some(index)
    }

    fn set(&mut self, offset: usize, value: usize) -> usize {
        debug_assert!(self.index_size != 0);
        debug_assert!(offset < self.len);

        let indices_per_u64 = self.indices_per_u64 as usize;
        let target_u64 = unsafe { self.storage.get_unchecked_mut(offset / indices_per_u64) };
        let target_offset = (offset % indices_per_u64) * self.index_size;
        let old_index = (*target_u64 >> target_offset) & self.mask;

        *target_u64 &= !(self.mask << target_offset);
        *target_u64 |= (value as u64) << target_offset;

        old_index
            .try_into()
            .expect("`old_index` should fit in a `usize`")
    }

    fn get(&self, offset: usize) -> usize {
        debug_assert!(offset < self.len);

        if self.index_size == 0 {
            return 0;
        }

        let indices_per_u64 = self.indices_per_u64 as usize;
        let target_u64 = unsafe { self.storage.get_unchecked(offset / indices_per_u64) };
        let target_offset = (offset % indices_per_u64) * self.index_size;

        ((*target_u64 >> target_offset) & self.mask)
            .try_into()
            .expect("return value should fit in a `usize`")
    }
}
