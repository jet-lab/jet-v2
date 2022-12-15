#![allow(missing_docs)] // FIXME

use std::{
    collections::{hash_map::DefaultHasher, HashSet, LinkedList},
    hash::{Hash, Hasher},
    sync::Arc,
};

use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct AsyncNoDupeQueue<T: Hash + Eq>(Arc<Mutex<NoDupeQueue<T>>>);

impl<T: Hash + Eq> AsyncNoDupeQueue<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(NoDupeQueue::new())))
    }

    pub async fn push(&self, item: T) {
        self.0.lock().await.push(item)
    }

    pub async fn pop(&self) -> Option<T> {
        self.0.lock().await.pop()
    }

    pub async fn push_many(&self, items: Vec<T>) {
        let mut inner = self.0.lock().await;
        for item in items {
            inner.push(item);
        }
    }

    pub async fn pop_many(&self, max: usize) -> Vec<T> {
        let mut inner = self.0.lock().await;
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

#[derive(Default)]
pub struct NoDupeQueue<T: Hash + Eq> {
    list: LinkedList<T>,
    set: HashSet<u64>,
}

impl<T: Hash + Eq> NoDupeQueue<T> {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
            set: HashSet::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        let key = hash(&item);
        if !self.set.contains(&key) {
            self.set.insert(key);
            self.list.push_back(item);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.list.pop_front().map(|item| {
            self.set.remove(&hash(&item));
            item
        })
    }
}

fn hash<T: Hash>(item: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}
