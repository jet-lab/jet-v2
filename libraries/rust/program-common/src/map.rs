use std::cell::RefMut;

use bytemuck::{Pod, Zeroable};
use thiserror::Error;

/// The initial bytes that mark the map state
pub const MAP_MAGIC: [u8; 4] = *b"jmap";

/// An error returned from a map operation
#[derive(Error, Debug)]
pub enum MapError {
    /// Attempting to insert into a full map
    #[error("map is full (size is {0})")]
    Full(usize),

    /// Attempting to load a map that doesn't match the expected format
    ///
    /// Implies the map data is either corrupt or was never initialized
    #[error("cannot load map with invalid data format")]
    InvalidFormat,
}

/// A general purpose map, suitable for use in on-chain account data.
///
/// Stores all data in a byte buffer, and the capacity of the map can be extended
/// by increasing the size of the buffer. This makes it useful for storing a dynamically
/// sized map inside Solana account data.
///
/// The map structure uses a 16 byte header, and each entry in the map requires an extra
/// 16 bytes of space for bookkeeping.
///
/// Limitations:
///     * The map capacity can only be increased by extending the backing buffer, it cannot be shrunk
pub struct Map<'a, K, V>
where
    K: Pod + AsRef<[u8]>,
    V: Pod,
{
    header: RefMut<'a, Header>,
    storage: RefMut<'a, [StorageNode<K, V>]>,
}

impl<'a, K, V> Map<'a, K, V>
where
    V: Pod,
    K: Pod + AsRef<[u8]> + std::fmt::Debug,
{
    /// The size of the map header
    pub const HEADER_SIZE: usize = std::mem::size_of::<Header>();

    /// The size of one entry in the map
    pub const ENTRY_SIZE: usize = std::mem::size_of::<StorageNode<K, V>>();

    /// Initialize a new map inside a buffer
    pub fn initialize(buffer: RefMut<'a, [u8]>) -> Self {
        let (buf_header, buf_nodes) = RefMut::map_split(buffer, |buffer| {
            buffer.split_at_mut(std::mem::size_of::<Header>())
        });

        let mut header = RefMut::map(buf_header, |header| {
            bytemuck::from_bytes_mut::<Header>(header)
        });
        let storage = RefMut::map(buf_nodes, |nodes| bytemuck::pod_align_to_mut(nodes).1);

        header.magic = MAP_MAGIC;

        Self { header, storage }
    }

    /// Load a map from a buffer
    pub fn from_buffer(buffer: RefMut<'a, [u8]>) -> Result<Self, MapError> {
        if &buffer[..4] != MAP_MAGIC {
            return Err(MapError::InvalidFormat);
        }

        let (buf_header, buf_nodes) = RefMut::map_split(buffer, |buffer| {
            buffer.split_at_mut(std::mem::size_of::<Header>())
        });

        let header = RefMut::map(buf_header, |header| {
            bytemuck::from_bytes_mut::<Header>(header)
        });
        let storage = RefMut::map(buf_nodes, |nodes| bytemuck::pod_align_to_mut(nodes).1);

        Ok(Self { header, storage })
    }

    /// Get the total number of entries in the map
    pub fn len(&self) -> usize {
        self.storage.len() - self.count_unallocated_leaves()
    }

    /// Get the maximium number of entries the map can hold with the current backing buffer
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Retrieve the value for a given key
    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<&V> {
        let node_ptr = self.find_leaf(key)?;
        let leaf = self.get_leaf(node_ptr);

        Some(&leaf.value)
    }

    /// Retrieve the mutable value for a given key
    pub fn get_mut(&mut self, key: impl AsRef<[u8]>) -> Option<&mut V> {
        let node_ptr = self.find_leaf(key)?;
        let leaf = self.get_leaf_mut(node_ptr);

        Some(&mut leaf.value)
    }

    /// Insert a new entry into the map
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, MapError> {
        let kb = key.as_ref();

        if self.header.root().is_none() {
            // empty, insert at root
            let node = self.allocate_leaf()?;
            let leaf = self.get_leaf_mut(node);

            leaf.key = key;
            leaf.value = value;
            self.header.root = node;

            return Ok(None);
        };

        let nearest = self.find_nearest_leaf(kb);
        let nearest_leaf = self.get_leaf_mut(nearest);
        let Some((diff_index, diff_bits)) = diff_index(nearest_leaf.key.as_ref(), kb) else {
            // key already exists in the map, replace the existing value
            return Ok(Some(std::mem::replace(&mut nearest_leaf.value, value)));
        };

        let inner_value = diff_value(diff_bits);
        let direction = (1 + (inner_value | nearest_leaf.key.as_ref()[diff_index]) as u32) >> 8;

        let new_leaf_node = self.allocate_leaf()?;
        let new_leaf = self.get_leaf_mut(new_leaf_node);

        new_leaf.key = key;
        new_leaf.value = value;

        let new_inner_node = self.allocate_inner()?;
        let new_inner = self.get_inner_mut(new_inner_node);

        new_inner.set_len(diff_index);
        new_inner.set_value(inner_value);
        new_inner.set_child_ptr(1 - direction, new_leaf_node);

        self.insert_inner(new_inner_node, &key, direction);

        Ok(None)
    }

    /// Remove a value from the map
    pub fn remove(&mut self, key: impl AsRef<[u8]> + std::fmt::Debug) -> Option<V> {
        let kb = key.as_ref();
        let mut p = self.header.root;
        let mut q = 0;
        let mut p_ref = None;
        let mut q_ref = None;

        if (p & PTR_SET_FLAG) == 0 {
            // map is empty
            return None;
        }

        while (p & INNER_NODE_FLAG) != 0 {
            q = p;
            q_ref = p_ref;

            let q_node = self.get_inner(q);
            let c = match q_node.len() {
                len if len < kb.len() => kb[len],
                _ => 0,
            };
            let direction = (1 + (q_node.value() | c) as u32) >> 8;

            p = q_node.child_ptr(direction);
            p_ref = Some((q, direction));
        }

        let p_node = self.get_leaf(p);
        if p_node.key.as_ref() != kb {
            // no such key in the map
            return None;
        }

        let value = p_node.value.clone();
        self.free_leaf(p);

        if q_ref.is_none() {
            // leaf parent is the root node
            match p_ref {
                None => self.header.root = 0,
                Some((parent, remove_dir)) => {
                    let node = self.get_inner(parent);
                    self.header.root = node.child_ptr(1 - remove_dir);
                }
            };

            return Some(value);
        }

        let (q_parent, q_direction) = q_ref.unwrap();
        let q_child = self.get_inner(q).child_ptr(1 - q_direction);
        let q_parent_node = self.get_inner_mut(q_parent);
        q_parent_node.set_child_ptr(q_direction, q_child);

        self.free_inner(q);

        Some(value)
    }

    /// Get an iterator over all the items in the map
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        Iter {
            map: &self,
            next: self.header.root().map(|r| vec![r]).unwrap_or_default(),
        }
    }

    fn insert_inner(&mut self, inner_ptr: u32, key: &K, direction: u32) {
        let kb = key.as_ref();
        let inner = &self.storage[(inner_ptr & NODE_ID_MASK) as usize].inner;
        let new_len = inner.len();
        let new_value = inner.value();

        let mut target = self.header.root;
        let mut to_update = None;

        loop {
            if (target & INNER_NODE_FLAG) == 0 {
                break;
            }

            let node = &self.storage[(target & NODE_ID_MASK) as usize].inner;
            //let new_len_v = new_len & node.value() as usize;

            if node.len() > new_len || (node.len() == new_len && node.value() > new_value) {
                break;
            }

            let c = match node.len() {
                len if len < kb.len() => kb[len],
                _ => 0,
            };

            let search_dir = (1 + (node.value() | c) as u32) >> 8;
            to_update = Some((target, search_dir));
            target = node.child_ptr(search_dir);
        }

        let inner = self.get_inner_mut(inner_ptr);
        inner.set_child_ptr(direction, target);

        match to_update {
            Some((target, search_dir)) => {
                let node = self.get_inner_mut(target);
                node.set_child_ptr(search_dir, inner_ptr);
            }
            None => {
                self.header.root = inner_ptr;
            }
        }
    }

    fn find_leaf(&self, key: impl AsRef<[u8]>) -> Option<NodePtr> {
        let kb = key.as_ref();
        let node_ptr = self.find_nearest_leaf(kb);
        let leaf = self.get_leaf(node_ptr);

        if leaf.key.as_ref() == kb {
            Some(node_ptr)
        } else {
            None
        }
    }

    fn find_nearest_leaf(&self, key: impl AsRef<[u8]>) -> NodePtr {
        let kb = key.as_ref();
        let Some(mut node_ptr) = self.header.root() else {
            unreachable!()
        };

        while (node_ptr & INNER_NODE_FLAG) != 0 {
            let node = self.get_inner(node_ptr);

            let c = match node.len() {
                len if len < kb.len() => kb[len],
                _ => 0,
            };

            let direction = (1 + (node.value() | c) as u32) >> 8;
            node_ptr = node.child_ptr(direction);
        }

        node_ptr
    }

    fn node(&self, ptr: NodePtr) -> &StorageNode<K, V> {
        let node_id = ptr & NODE_ID_MASK;
        &self.storage[node_id as usize]
    }

    fn node_mut(&mut self, ptr: NodePtr) -> &mut StorageNode<K, V> {
        let node_id = ptr & NODE_ID_MASK;
        &mut self.storage[node_id as usize]
    }

    fn get_inner(&self, ptr: NodePtr) -> &InternalNode {
        debug_assert!(ptr & INNER_NODE_FLAG != 0);
        &self.node(ptr).inner
    }

    fn get_inner_mut(&mut self, ptr: NodePtr) -> &mut InternalNode {
        debug_assert!(ptr & INNER_NODE_FLAG != 0);
        &mut self.node_mut(ptr).inner
    }

    fn get_leaf(&self, ptr: NodePtr) -> &LeafNode<K, V> {
        debug_assert!(ptr & INNER_NODE_FLAG == 0);
        &self.node(ptr).leaf
    }

    fn get_leaf_mut(&mut self, ptr: NodePtr) -> &mut LeafNode<K, V> {
        debug_assert!(ptr & INNER_NODE_FLAG == 0);
        &mut self.node_mut(ptr).leaf
    }

    fn allocate_leaf(&mut self) -> Result<NodePtr, MapError> {
        let next_ptr = self.header.next_free_leaf;
        let next_id = (next_ptr & NODE_ID_MASK) as usize;

        if next_id == self.storage.len() {
            return Err(MapError::Full(self.storage.len()));
        }

        let node = &mut self.storage[next_id as usize];
        self.header.next_free_leaf = node.header.next_free_leaf;

        if (self.header.next_free_leaf & PTR_SET_FLAG) == 0 {
            self.header.next_free_leaf = next_ptr + 1;
        }

        node.header = StorageNodeHeader::zeroed();

        Ok(next_ptr | PTR_SET_FLAG)
    }

    fn allocate_inner(&mut self) -> Result<NodePtr, MapError> {
        let next_ptr = self.header.next_free_inner;
        let next_id = (next_ptr & NODE_ID_MASK) as usize;

        if next_id == self.storage.len() {
            return Err(MapError::Full(self.storage.len()));
        }

        let node = &mut self.storage[next_id as usize];
        self.header.next_free_inner = node.header.next_free_inner;

        if (self.header.next_free_inner & PTR_SET_FLAG) == 0 {
            self.header.next_free_inner = next_ptr + 1;
        }

        node.header = StorageNodeHeader::zeroed();

        Ok(next_ptr | PTR_SET_FLAG | INNER_NODE_FLAG)
    }

    fn free_leaf(&mut self, ptr: NodePtr) {
        debug_assert!(ptr & INNER_NODE_FLAG == 0);

        let node_id = ptr & NODE_ID_MASK;
        let node = &mut self.storage[node_id as usize];

        node.leaf = LeafNode::zeroed();
        node.header.next_free_leaf = self.header.next_free_leaf;
        self.header.next_free_leaf = ptr;
    }

    fn count_unallocated_leaves(&self) -> usize {
        let mut count = 0;
        let mut ptr = self.header.next_free_leaf;

        while (ptr & PTR_SET_FLAG) != 0 {
            count += 1;
            ptr = self.node(ptr).header.next_free_leaf;
        }

        count += self.storage.len() - (ptr & NODE_ID_MASK) as usize;
        count
    }

    fn free_inner(&mut self, ptr: NodePtr) {
        debug_assert!(ptr & INNER_NODE_FLAG != 0);

        let node_id = ptr & NODE_ID_MASK;
        let node = &mut self.storage[node_id as usize];

        node.inner = InternalNode::zeroed();
        node.header.next_free_inner = self.header.next_free_leaf;
        self.header.next_free_inner = ptr;
    }

    // #[cfg(test)]
    // fn traverse(&self, mut f: impl FnMut(NodePtr)) {
    //     let root = match self.header.root() {
    //         Some(root) => root,
    //         None => return,
    //     };

    //     let mut unvisited = vec![root];

    //     while let Some(next) = unvisited.pop() {
    //         f(next);

    //         if next & INNER_NODE_FLAG != 0 {
    //             let node = self.get_inner(next);

    //             unvisited.push(node.child_ptr(0u32));
    //             unvisited.push(node.child_ptr(1u32));
    //         }
    //     }
    // }

    // #[cfg(test)]
    // fn debug_structure(&self) {
    //     self.traverse(|ptr| {
    //         if ptr & INNER_NODE_FLAG != 0 {
    //             let node = self.get_inner(ptr);
    //             println!(r#""{:x}" -> "{:x}""#, ptr, node.child_ptr(0u32));
    //             println!(r#""{:x}" -> "{:x}""#, ptr, node.child_ptr(1u32));
    //         }
    //     });
    // }
}

pub struct Iter<'r, 's, K, V>
where
    K: Pod + AsRef<[u8]> + std::fmt::Debug,
    V: Pod,
{
    map: &'r Map<'s, K, V>,
    next: Vec<NodePtr>,
}

impl<'r, 's, K, V> Iterator for Iter<'r, 's, K, V>
where
    K: Pod + AsRef<[u8]> + std::fmt::Debug,
    V: Pod,
{
    type Item = (&'r K, &'r V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.next.pop() {
            if next & INNER_NODE_FLAG != 0 {
                let node = self.map.get_inner(next);

                self.next.push(node.child_ptr(0u32));
                self.next.push(node.child_ptr(1u32));
            } else {
                let node = self.map.get_leaf(next);
                return Some((&node.key, &node.value));
            }
        }

        None
    }
}

const NODE_ID_MASK: u32 = 0xFFFF;
const PTR_MASK: u32 = 0xFFFFFF;
const PTR_SET_FLAG: u32 = 1 << 18;
const INNER_NODE_FLAG: u32 = 1 << 17;

type NodePtr = u32;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct Header {
    magic: [u8; 4],
    root: NodePtr,
    next_free_inner: NodePtr,
    next_free_leaf: NodePtr,
}

impl Header {
    fn root(&self) -> Option<NodePtr> {
        if self.root & PTR_SET_FLAG == 0 {
            return None;
        }

        Some(self.root)
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct InternalNode {
    children: [NodePtr; 2],
}

impl InternalNode {
    fn len(&self) -> usize {
        (self.children[0] >> 24) as usize
    }

    fn set_len(&mut self, len: usize) {
        debug_assert!(len < 256);
        self.children[0] |= (len as u32) << 24;
    }

    fn value(&self) -> u8 {
        (self.children[1] >> 24) as u8
    }

    fn set_value(&mut self, value: u8) {
        self.children[1] |= (value as u32) << 24;
    }

    fn child_ptr(&self, direction: impl Into<u64>) -> NodePtr {
        let direction = direction.into() as usize;

        debug_assert!(direction < 2);
        self.children[direction] & PTR_MASK
    }

    fn set_child_ptr(&mut self, direction: impl Into<u64>, ptr: NodePtr) {
        let direction = direction.into() as usize;

        debug_assert!(direction < 2);
        self.children[direction] = (self.children[direction] & (0xFF << 24)) | ptr;
    }
}

impl std::fmt::Debug for InternalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.len();
        let value = self.value();
        let children = self
            .children
            .iter()
            .map(|ptr| ptr & PTR_MASK)
            .collect::<Vec<_>>();

        f.debug_struct("InternalNode")
            .field("len", &len)
            .field("value", &value)
            .field("children", &children)
            .finish()
    }
}

#[repr(C)]
#[derive(Zeroable, Clone, Copy)]
struct LeafNode<K, V>
where
    K: Pod,
    V: Pod,
{
    key: K,
    value: V,
}

unsafe impl<K: Pod, V: Pod> Pod for LeafNode<K, V> {}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct StorageNodeHeader {
    next_free_inner: NodePtr,
    next_free_leaf: NodePtr,
}

#[repr(C, align(8))]
#[derive(Zeroable, Clone, Copy)]
struct StorageNode<K, V>
where
    K: Pod,
    V: Pod,
{
    header: StorageNodeHeader,
    inner: InternalNode,
    leaf: LeafNode<K, V>,
}

unsafe impl<K: Pod, V: Pod> Pod for StorageNode<K, V> {}

fn diff_index(key0: &[u8], key1: &[u8]) -> Option<(usize, u32)> {
    let mut index = 0;

    for i in 0..key1.len() {
        index = i;

        let kb_0 = key0.get(i).cloned().unwrap_or_default();
        let kb_1 = key1[i];

        if kb_0 != kb_1 {
            return Some((index, (kb_0 ^ kb_1).into()));
        }
    }

    let kb_0 = key0.get(index).cloned().unwrap_or_default();
    if kb_0 != 0 {
        return Some((index, kb_0.into()));
    }

    None
}

fn diff_value(mut bits: u32) -> u8 {
    bits |= bits >> 1;
    bits |= bits >> 2;
    bits |= bits >> 4;
    ((bits & !(bits >> 1)) ^ 0xFF) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use std::cell::RefCell;

    #[test]
    fn can_insert_remove_one() {
        let storage = [0u8; 256];
        let buffer = RefCell::new(storage);
        let mut map = Map::initialize(buffer.borrow_mut());
        let key = Pubkey::default();

        map.insert(key, 42).unwrap();
        map.remove(key).unwrap();
    }

    #[test]
    fn can_find_entry_in_seq_2() {
        let storage = [0u8; 2048];
        let buffer = RefCell::new(storage);
        let mut map = Map::initialize(buffer.borrow_mut());
        let mut keys = vec![];

        for i in 0..2 {
            let key_buf = [i; 32];
            let key = Pubkey::from(key_buf);

            assert!(map.insert(key, i).unwrap().is_none());
            keys.push(key);
        }

        assert!(map.get(&keys[0]).is_some());
    }

    #[test]
    fn correct_len_20() {
        let storage = [0u8; 2048];
        let buffer = RefCell::new(storage);
        let mut map = Map::initialize(buffer.borrow_mut());

        for i in 0..20 {
            let key_buf = [i; 32];
            let key = Pubkey::from(key_buf);

            assert!(map.insert(key, i).unwrap().is_none());
        }

        assert_eq!(map.len(), 20);
        assert_eq!(map.capacity(), 36)
    }

    #[test]
    fn can_insert_remove_seq_20() {
        let storage = [0u8; 2048];
        let buffer = RefCell::new(storage);
        let mut map = Map::initialize(buffer.borrow_mut());
        let mut keys = vec![];

        for i in 0..20 {
            let key_buf = [i; 32];
            let key = Pubkey::from(key_buf);

            assert!(map.insert(key, i).unwrap().is_none());
            keys.push(key);
        }

        for key in keys {
            assert!(map.remove(key).is_some(), "key not removed: {:?}", key);
        }
    }

    #[test]
    fn can_iter() {
        let storage = [0u8; 16384];
        let buffer = RefCell::new(storage);
        let mut map = Map::initialize(buffer.borrow_mut());
        let mut keys = vec![];

        for i in 0..128 {
            let key_buf = [i; 32];
            let key = Pubkey::from(key_buf);

            assert!(map.insert(key, i).unwrap().is_none());
            keys.push(key);
        }

        for (k, (ik, _)) in keys.iter().rev().zip(map.iter()) {
            assert_eq!(k, ik);
        }
    }
}
