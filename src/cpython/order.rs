//! CPython 3.12 set layout and difference behavior, derived from
//! Objects/setobject.c in CPython v3.12.11 (see LICENSE-CPYTHON).

use std::collections::HashSet;
use std::iter::FilterMap;

const INITIAL_SIZE: usize = 8;
const LINEAR_PROBES: usize = 9;
const PERTURB_SHIFT: u32 = 5;

#[derive(Clone)]
pub(super) enum Slot<T> {
    Empty,
    Deleted,
    Key(T, usize),
}

#[derive(Clone)]
pub(super) struct Order<T> {
    slots: Vec<Slot<T>>,
    filled: usize,
}

pub(super) type Keys<'a, T> =
    FilterMap<std::slice::Iter<'a, Slot<T>>, fn(&'a Slot<T>) -> Option<&'a T>>;
pub(super) type IntoKeys<T> = FilterMap<std::vec::IntoIter<Slot<T>>, fn(Slot<T>) -> Option<T>>;

impl<T> Order<T> {
    pub(super) fn new() -> Self {
        Self {
            slots: Vec::new(),
            filled: 0,
        }
    }

    pub(super) fn iter(&self) -> Keys<'_, T> {
        self.slots.iter().filter_map(|slot| match slot {
            Slot::Key(key, _) => Some(key),
            _ => None,
        })
    }

    pub(super) fn into_keys(self) -> IntoKeys<T> {
        self.slots.into_iter().filter_map(|slot| match slot {
            Slot::Key(key, _) => Some(key),
            _ => None,
        })
    }
}

impl<T: std::hash::Hash + Eq + Clone> Order<T> {
    pub(super) fn insert(&mut self, key: T, hash: usize, used: usize) {
        if self.slots.is_empty() {
            self.resize(0);
        }
        let mask = self.slots.len() - 1;
        let mut deleted = None;
        for index in probes(hash, mask) {
            match self.slots[index] {
                Slot::Deleted => deleted = Some(index),
                Slot::Empty => {
                    // CPython reuses the last dummy visited before an empty slot.
                    self.slots[deleted.unwrap_or(index)] = Slot::Key(key, hash);
                    if deleted.is_none() {
                        self.filled += 1;
                        if self.filled * 5 >= mask * 3 {
                            self.resize(resize_threshold(used));
                        }
                    }
                    return;
                }
                Slot::Key(..) => {}
            }
        }
        unreachable!();
    }

    pub(super) fn remove(&mut self, key: &T, hash: usize) {
        for index in probes(hash, self.slots.len() - 1) {
            if matches!(&self.slots[index], Slot::Key(existing, h) if *h == hash && key == existing)
            {
                self.slots[index] = Slot::Deleted;
                return;
            }
        }
        unreachable!();
    }

    pub(super) fn difference(&self, lhs_len: usize, rhs_len: usize, result: &HashSet<T>) -> Self {
        if lhs_len >> 2 > rhs_len {
            // CPython copies the left table via set_merge, then removes keys
            // and compacts if too many dummy slots remain.
            let mut copy = self.copy_for_difference(lhs_len);
            for slot in &mut copy.slots {
                if matches!(slot, Slot::Key(key, _) if !result.contains(key)) {
                    *slot = Slot::Deleted;
                }
            }
            if copy.filled - result.len() > (copy.slots.len() - 1) / 4 {
                copy.resize(resize_threshold(result.len()));
            }
            return copy;
        }

        let mut order = Self::new();
        let mut used = 0;
        for slot in &self.slots {
            if let Slot::Key(key, hash) = slot
                && result.contains(key)
            {
                used += 1;
                order.insert(key.clone(), *hash, used);
            }
        }
        order
    }

    fn copy_for_difference(&self, used: usize) -> Self {
        let mut copy = Self::new();
        let min_used = if used * 5 >= (INITIAL_SIZE - 1) * 3 {
            used * 2
        } else {
            0
        };
        copy.resize(min_used);
        copy.filled = used;
        if copy.slots.len() == self.slots.len() && self.filled == used {
            copy.slots.clone_from(&self.slots);
        } else {
            for slot in &self.slots {
                if let Slot::Key(key, hash) = slot {
                    insert_clean(&mut copy.slots, key.clone(), *hash);
                }
            }
        }
        copy
    }

    fn resize(&mut self, min_used: usize) {
        let size = (min_used + 1).next_power_of_two().max(INITIAL_SIZE);
        let mut slots = vec![Slot::Empty; size];
        self.filled = 0;
        for slot in self.slots.drain(..) {
            if let Slot::Key(key, hash) = slot {
                insert_clean(&mut slots, key, hash);
                self.filled += 1;
            }
        }
        self.slots = slots;
    }
}

fn resize_threshold(used: usize) -> usize {
    if used > 50_000 { used * 2 } else { used * 4 }
}

fn insert_clean<T>(slots: &mut [Slot<T>], key: T, hash: usize) {
    let index = probes(hash, slots.len() - 1)
        .find(|&index| matches!(slots[index], Slot::Empty))
        .expect("CPython layout must have an empty slot");
    slots[index] = Slot::Key(key, hash);
}

fn probes(hash: usize, mask: usize) -> impl Iterator<Item = usize> {
    std::iter::successors(Some((hash & mask, hash)), move |&(base, perturb)| {
        let perturb = perturb >> PERTURB_SHIFT;
        let base = base.wrapping_mul(5).wrapping_add(1).wrapping_add(perturb) & mask;
        Some((base, perturb))
    })
    .flat_map(move |(base, _)| {
        if base + LINEAR_PROBES <= mask {
            base..=base + LINEAR_PROBES
        } else {
            base..=base
        }
    })
}
