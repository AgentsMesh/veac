use std::collections::{BTreeMap, VecDeque};
use std::mem::size_of;
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct BoundedCache<K, V> {
    entry_limit: usize,
    byte_limit: usize,
    values: BTreeMap<K, Entry<V>>,
    order: VecDeque<K>,
    retained_bytes: usize,
}

#[derive(Debug)]
struct Entry<V> {
    retained_bytes: usize,
    value: Arc<V>,
}

pub(super) struct InsertOutcome<V> {
    pub value: Arc<V>,
    pub inserted: bool,
    pub bypassed: bool,
    pub evictions: u64,
}

impl<K: Ord + Clone, V> BoundedCache<K, V> {
    pub(super) fn new(entry_limit: usize, byte_limit: usize) -> Self {
        Self {
            entry_limit,
            byte_limit,
            values: BTreeMap::new(),
            order: VecDeque::new(),
            retained_bytes: 0,
        }
    }

    pub(super) fn get(&self, key: &K) -> Option<Arc<V>> {
        self.values.get(key).map(|entry| Arc::clone(&entry.value))
    }

    pub(super) fn insert(&mut self, key: K, bytes: usize, value: Arc<V>) -> InsertOutcome<V> {
        if let Some(existing) = self.get(&key) {
            return outcome(existing, false, false, 0);
        }
        if self.entry_limit == 0 || self.byte_limit == 0 || bytes > self.byte_limit {
            return outcome(value, false, true, 0);
        }
        let bytes = match entry_overhead::<K, V>().and_then(|overhead| bytes.checked_add(overhead))
        {
            Some(value) if value <= self.byte_limit => value,
            _ => return outcome(value, false, true, 0),
        };
        if self.retained_bytes.checked_add(bytes).is_none() {
            return outcome(value, false, true, 0);
        }
        let mut evictions: u64 = 0;
        while self.values.len() >= self.entry_limit
            || exceeds(self.retained_bytes, bytes, self.byte_limit)
        {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(removed) = self.values.remove(&oldest) {
                self.retained_bytes -= removed.retained_bytes;
                evictions = evictions.saturating_add(1);
            }
        }
        self.retained_bytes = self
            .retained_bytes
            .checked_add(bytes)
            .expect("cache admission checked retained byte arithmetic");
        self.order.push_back(key.clone());
        self.values.insert(
            key,
            Entry {
                retained_bytes: bytes,
                value: Arc::clone(&value),
            },
        );
        outcome(value, true, false, evictions)
    }

    pub(super) fn clear(&mut self) {
        self.values = BTreeMap::new();
        self.order = VecDeque::new();
        self.retained_bytes = 0;
    }

    pub(super) fn len(&self) -> usize {
        self.values.len()
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}

// The cache owns one map key, one FIFO key clone, one value entry and the
// allocator/container shells. Dynamic key/value payloads are charged by the
// caller; this fixed charge covers their inline ownership handles.
fn entry_overhead<K, V>() -> Option<usize> {
    // BTreeMap/VecDeque do not expose allocator capacities. Charge a stable
    // upper bound for their nodes so byte budgets remain conservative.
    size_of::<K>()
        .checked_mul(2)?
        .checked_add(size_of::<Entry<V>>())?
        .checked_add(512)
}

fn exceeds(current: usize, added: usize, limit: usize) -> bool {
    current.checked_add(added).is_none_or(|total| total > limit)
}

fn outcome<V>(value: Arc<V>, inserted: bool, bypassed: bool, evictions: u64) -> InsertOutcome<V> {
    InsertOutcome {
        value,
        inserted,
        bypassed,
        evictions,
    }
}

#[cfg(test)]
#[path = "bounded_cache/tests.rs"]
mod tests;
