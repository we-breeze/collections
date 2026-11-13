//! Breeze collections with build-time selection of iteration behavior.

use std::fmt;
use std::hash::Hash;
use std::ops::Sub;

#[cfg(all(brz_cpython_set, not(target_pointer_width = "64")))]
compile_error!("the CPython set backend requires a 64-bit target");

#[cfg(brz_cpython_set)]
mod cpython;
#[cfg(brz_cpython_set)]
use cpython::CPythonSet as InnerSet;
#[cfg(not(brz_cpython_set))]
use std::collections::HashSet as InnerSet;

#[cfg(brz_cpython_set)]
use CPythonHash as SetKey;
#[cfg(not(brz_cpython_set))]
use Hash as SetKey;

/// Hash a Rust key as its equivalent key in 64-bit CPython.
///
/// Keys that compare equal in Rust must return equal hashes, and their Rust
/// equality must agree with the equality of the corresponding Python keys.
/// Implementations must be stable for as long as a key is in a set.
pub trait CPythonHash: Hash + Clone {
    /// Returns the signed CPython hash. CPython reserves `-1` and uses `-2`
    /// for keys that would otherwise hash to that value.
    fn cpython_hash(&self) -> i64;
}

impl CPythonHash for i64 {
    fn cpython_hash(&self) -> i64 {
        const MODULUS: i64 = (1 << 61) - 1;
        match self % MODULUS {
            -1 => -2,
            hash => hash,
        }
    }
}

/// A hash set backed by the standard library.
///
/// Enable `cpython-set` or set `BRZ_COLLECTIONS_BACKEND=cpython` at build time
/// to enable CPython iteration order. In that build, key operations require
/// `T: CPythonHash + Eq`. The default build uses `T: Hash + Eq`.
#[derive(Clone)]
pub struct BrzHashSet<T>(InnerSet<T>);

impl<T> BrzHashSet<T> {
    /// Creates an empty set.
    pub fn new() -> Self {
        Self(InnerSet::new())
    }

    /// Reserves capacity for the Rust member table. In a CPython build, the
    /// logical order starts with the same empty table as Python's `set()`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(InnerSet::with_capacity(capacity))
    }

    /// Returns the number of distinct keys.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Removes all elements and resets the iteration order.
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl<T: Eq + SetKey> BrzHashSet<T> {
    /// Visits each member in the order selected at build time.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter(self.0.iter())
    }

    /// Returns whether the set contains the key.
    pub fn contains(&self, key: &T) -> bool {
        self.0.contains(key)
    }

    /// Inserts a key, returning whether it was newly added.
    pub fn insert(&mut self, key: T) -> bool {
        self.0.insert(key)
    }

    /// Removes a key, returning whether it was present.
    pub fn remove(&mut self, key: &T) -> bool {
        self.0.remove(key)
    }
}

impl<T> Default for BrzHashSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + SetKey + fmt::Debug> fmt::Debug for BrzHashSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T: Eq + SetKey> PartialEq for BrzHashSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq + SetKey> Eq for BrzHashSet<T> {}

impl<T: Eq + SetKey> FromIterator<T> for BrzHashSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: Eq + SetKey> Extend<T> for BrzHashSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: Eq + SetKey + Clone> Sub for &BrzHashSet<T> {
    type Output = BrzHashSet<T>;

    fn sub(self, other: Self) -> Self::Output {
        BrzHashSet(&self.0 - &other.0)
    }
}

#[cfg(brz_cpython_set)]
use cpython::{IntoIter as InnerIntoIter, Iter as InnerIter};
#[cfg(not(brz_cpython_set))]
use std::collections::hash_set::{IntoIter as InnerIntoIter, Iter as InnerIter};

/// Borrowed iterator over a `BrzHashSet`.
pub struct Iter<'a, T>(InnerIter<'a, T>);

impl<'a, T: Eq + SetKey> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: Eq + SetKey> ExactSizeIterator for Iter<'_, T> {}
impl<T: Eq + SetKey> std::iter::FusedIterator for Iter<'_, T> {}

impl<'a, T: Eq + SetKey> IntoIterator for &'a BrzHashSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Owning iterator over a `BrzHashSet`.
pub struct IntoIter<T>(InnerIntoIter<T>);

impl<T: Eq + SetKey> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: Eq + SetKey> ExactSizeIterator for IntoIter<T> {}
impl<T: Eq + SetKey> std::iter::FusedIterator for IntoIter<T> {}

impl<T: Eq + SetKey> IntoIterator for BrzHashSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}
