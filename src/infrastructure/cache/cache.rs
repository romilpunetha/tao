use lru::LruCache;
use std::num::NonZeroUsize;

pub struct Cache<K, V> {
    inner: LruCache<K, V>,
}

impl<K: std::hash::Hash + Eq, V> Cache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Cache {
            inner: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.inner.put(key, value);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.pop(key)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
