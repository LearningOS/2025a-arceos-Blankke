//! Collection types.
//! 
//! This module provides a `HashMap` implementation using hashbrown for no_std environments,
//! and re-exports other collection types from alloc.

#[cfg(feature = "alloc")]
pub use alloc::collections::*;

use core::hash::{BuildHasher, Hasher};

/// A simple hasher that uses the axhal random function for seeding
#[cfg(feature = "alloc")]
pub struct AxeosHasher {
    state: u64,
}

#[cfg(feature = "alloc")]
impl Default for AxeosHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl AxeosHasher {
    pub fn new() -> Self {
        // Use axhal's random function for initial state
        let random_seed = axhal::misc::random() as u64;
        Self {
            state: random_seed + 0x9e3779b9,
        }
    }
}

#[cfg(feature = "alloc")]
impl Hasher for AxeosHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state = self.state * 31 + byte as u64;
        }
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

/// A hash builder that creates AxeosHasher instances
#[cfg(feature = "alloc")]
pub struct AxeosHashBuilder;

#[cfg(feature = "alloc")]
impl Default for AxeosHashBuilder {
    fn default() -> Self {
        Self
    }
}

#[cfg(feature = "alloc")]
impl BuildHasher for AxeosHashBuilder {
    type Hasher = AxeosHasher;

    fn build_hasher(&self) -> Self::Hasher {
        AxeosHasher::new()
    }
}

/// A newtype wrapper for HashMap with AxeosHashBuilder
#[cfg(feature = "alloc")]
pub struct HashMap<K, V>(hashbrown::HashMap<K, V, AxeosHashBuilder>);

/// A newtype wrapper for HashSet with AxeosHashBuilder  
#[cfg(feature = "alloc")]
pub struct HashSet<T>(hashbrown::HashSet<T, AxeosHashBuilder>);

#[cfg(feature = "alloc")]
impl<K, V> HashMap<K, V> {
    /// Creates an empty `HashMap`.
    pub fn new() -> Self {
        Self(hashbrown::HashMap::with_hasher(AxeosHashBuilder::default()))
    }

    /// Creates an empty `HashMap` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(hashbrown::HashMap::with_capacity_and_hasher(capacity, AxeosHashBuilder::default()))
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, k: K, v: V) -> Option<V> 
    where
        K: core::hash::Hash + Eq,
    {
        self.0.insert(k, v)
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: core::borrow::Borrow<Q> + core::hash::Hash + Eq,
        Q: core::hash::Hash + Eq + ?Sized,
    {
        self.0.get(k)
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    pub fn iter(&self) -> hashbrown::hash_map::Iter<'_, K, V> {
        self.0.iter()
    }
}

#[cfg(feature = "alloc")]
impl<T> HashSet<T> {
    /// Creates an empty `HashSet`.
    pub fn new() -> Self {
        Self(hashbrown::HashSet::with_hasher(AxeosHashBuilder::default()))
    }

    /// Creates an empty `HashSet` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(hashbrown::HashSet::with_capacity_and_hasher(capacity, AxeosHashBuilder::default()))
    }
}