#![allow(missing_docs)] // FIXME

use std::{
    collections::{hash_map::DefaultHasher, HashSet, LinkedList},
    hash::{Hash, Hasher},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{pin_mut, Future, Stream};
use tokio::sync::Mutex;

#[derive(Clone)]
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
}

impl<T: Hash + Eq> Stream for AsyncNoDupeQueue<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let fut = self.pop();
        pin_mut!(fut);
        Future::poll(fut, cx)
    }
}

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
