//! Simple thread safe hashmap

use std::{collections::HashMap, hash::Hash, ops::Deref, sync::RwLock};

/// Thread safe hashmap
#[derive(Debug)]
pub struct RwHashMap<K, V>(pub(crate) RwLock<HashMap<K, V>>);

impl<K, V> RwHashMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    /// Create new, empty collection
    pub fn new() -> Self {
        RwHashMap(RwLock::new(HashMap::new()))
    }

    /// Lookup the key in hashmap
    pub fn get(&self, key: &K) -> Option<V> {
        self.0.read().unwrap().deref().get(key).cloned()
    }

    /// Insert a key-value pair to the hashmap
    pub fn insert(&self, key: K, value: V) {
        let mut cache = self.0.write().unwrap();
        cache.insert(key, value);
    }

    /// Get number of elements in the hashmap
    pub fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    /// Remove all elements from hashmap
    pub fn clear(&self) {
        let mut cache = self.0.write().unwrap();
        cache.clear();
    }
}
