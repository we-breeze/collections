//! Rust membership with a separate CPython iteration layout.

mod order;

use crate::CPythonHash;
use order::Order;
use std::collections::HashSet;
use std::iter::FusedIterator;
use std::ops::Sub;

#[derive(Clone)]
pub(super) struct CPythonSet<T> {
    members: HashSet<T>,
    order: Order<T>,
}

impl<T> CPythonSet<T> {
    pub(super) fn new() -> Self {
        Self::with_capacity(0)
    }

    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            members: HashSet::with_capacity(capacity),
            order: Order::new(),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.members.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub(super) fn clear(&mut self) {
        self.members.clear();
        self.order = Order::new();
    }
}

impl<T: CPythonHash + Eq> CPythonSet<T> {
    pub(super) fn contains(&self, key: &T) -> bool {
        self.members.contains(key)
    }

    pub(super) fn insert(&mut self, key: T) -> bool {
        let order_key = key.clone();
        let hash = key.cpython_hash() as usize;
        if self.members.insert(key) {
            self.order.insert(order_key, hash, self.members.len());
            true
        } else {
            false
        }
    }

    pub(super) fn remove(&mut self, key: &T) -> bool {
        let hash = key.cpython_hash() as usize;
        if let Some(member) = self.members.take(key) {
            self.order.remove(key, hash);
            drop(member);
            true
        } else {
            false
        }
    }

    pub(super) fn iter(&self) -> Iter<'_, T> {
        Iter {
            members: &self.members,
            keys: self.order.iter(),
            remaining: self.len(),
        }
    }

    pub(super) fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            members: self.members,
            keys: self.order.into_keys(),
        }
    }
}

impl<T: CPythonHash + Eq> PartialEq for CPythonSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.members == other.members
    }
}

impl<T: CPythonHash + Eq> FromIterator<T> for CPythonSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl<T: CPythonHash + Eq> Extend<T> for CPythonSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for key in iter {
            self.insert(key);
        }
    }
}

impl<T: CPythonHash + Eq> Sub for &CPythonSet<T> {
    type Output = CPythonSet<T>;

    fn sub(self, other: Self) -> Self::Output {
        let members = &self.members - &other.members;
        let order = self.order.difference(self.len(), other.len(), &members);
        CPythonSet { members, order }
    }
}

pub(super) struct Iter<'a, T> {
    members: &'a HashSet<T>,
    keys: order::Keys<'a, T>,
    remaining: usize,
}

impl<'a, T: CPythonHash + Eq> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys.next()?;
        self.remaining -= 1;
        Some(self.members.get(key).expect("order key missing from set"))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T: CPythonHash + Eq> ExactSizeIterator for Iter<'_, T> {}
impl<T: CPythonHash + Eq> FusedIterator for Iter<'_, T> {}

pub(super) struct IntoIter<T> {
    members: HashSet<T>,
    keys: order::IntoKeys<T>,
}

impl<T: CPythonHash + Eq> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let key = self.keys.next()?;
        Some(self.members.take(&key).expect("order key missing from set"))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.members.len();
        (remaining, Some(remaining))
    }
}

impl<T: CPythonHash + Eq> ExactSizeIterator for IntoIter<T> {}
impl<T: CPythonHash + Eq> FusedIterator for IntoIter<T> {}
