use std::{collections::HashMap, hash::Hash};

pub(crate) struct BBStoreBackend<K, V> {
    mem: HashMap<K, V>,
}

impl<K, V> Default for BBStoreBackend<K, V> {
    fn default() -> Self {
        Self {
            mem: HashMap::default(),
        }
    }
}

impl<K: Eq + Hash, V> BBStoreBackend<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.mem.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.mem.get(key)
    }
}
