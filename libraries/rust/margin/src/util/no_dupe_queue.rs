use std::{
    collections::{hash_map::DefaultHasher, HashSet, LinkedList},
    hash::{Hash, Hasher},
    sync::Arc,
};

use tokio::sync::RwLock;

/// Atomic version of NoDupeQueue that uses async tokio mutexes.
#[derive(Clone)]
pub struct AsyncNoDupeQueue<T: Hash + Eq>(Arc<RwLock<NoDupeQueue<T>>>);

impl<T: Hash + Eq> AsyncNoDupeQueue<T> {
    /// returns a new empty queue
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(NoDupeQueue::new())))
    }

    /// Acquires and releases the lock for a single push
    pub async fn push(&self, item: T) {
        self.0.write().await.push(item)
    }

    /// Acquires and releases the lock for a single pop
    pub async fn pop(&self) -> Option<T> {
        self.0.write().await.pop()
    }

    /// There are no items in the queue
    pub async fn is_empty(&self) -> bool {
        self.0.read().await.is_empty()
    }

    /// Number of items in the queue
    pub async fn len(&self) -> usize {
        self.0.read().await.len()
    }

    /// Adds many while acquiring the lock only once
    pub async fn push_many(&self, items: Vec<T>) {
        let mut inner = self.0.write().await;
        for item in items {
            inner.push(item);
        }
    }

    /// Pops many while acquiring the lock only once
    pub async fn pop_many(&self, max: usize) -> Vec<T> {
        let mut inner = self.0.write().await;
        let mut ret = vec![];
        for _ in 0..max {
            if let Some(item) = inner.pop() {
                ret.push(item);
            } else {
                break;
            }
        }
        ret
    }
}

impl<T: Hash + Eq> Default for AsyncNoDupeQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A deduplicated queue. If you attempt to add a duplicate of a value that is
/// already in the queue, it will simply not be added and the method will return
/// successfully, because it assumes that you are satisfied that it already
/// exists.
#[derive(Default)]
pub struct NoDupeQueue<T: Hash + Eq> {
    list: LinkedList<T>,
    set: HashSet<u64>,
}

impl<T: Hash + Eq> NoDupeQueue<T> {
    /// returns a new empty queue
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
            set: HashSet::new(),
        }
    }

    /// Adds an item to the back of the queue if it is not already present in
    /// the queue.
    pub fn push(&mut self, item: T) {
        let key = hash(&item);
        if !self.set.contains(&key) {
            self.set.insert(key);
            self.list.push_back(item);
        }
    }

    /// Removes the first item from the front of the queue and returns it.
    pub fn pop(&mut self) -> Option<T> {
        self.list.pop_front().map(|item| {
            self.set.remove(&hash(&item));
            item
        })
    }

    /// Number of items in the queue
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// There are no items in the queue
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}

fn hash<T: Hash>(item: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}
