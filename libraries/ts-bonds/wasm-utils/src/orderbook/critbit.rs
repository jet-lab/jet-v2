#![allow(dead_code)]

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
// A Slab contains the data for a slab header and two type-split arrays of inner nodes and leaves arranger in a critbit tree
// whose leaves contain data referencing an order of the orderbook.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
#[repr(C)]
pub struct CallbackInfo {
    /// The Pubkey of the user account that submit the order
    pub orderbook_account_key: [u8; 32],
    /// The order tag is generated by the program when submitting orders to the book
    /// Used to seed and track PDAs such as `Obligation`
    pub order_tag: [u8; 16],
    /// Pubkey of the account that will recieve the event information
    pub adapter_account_key: [u8; 32],
    /// configuration used by callback execution
    pub flags: u8,
    _reserved: [u8; 14],
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
#[repr(u8)]
/// Warning: the account tags are bitshifted to allow for standard tag usage in the program using the aob.
pub enum AccountTag {
    Uninitialized,
    Market = 1 << 7,
    EventQueue,
    Bids,
    Asks,
    Disabled,
}

#[doc(hidden)]
pub type IoError = std::io::Error;

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SlabHeader {
    leaf_free_list_len: u32,
    leaf_free_list_head: u32,
    leaf_bump_index: u32,

    inner_node_free_list_len: u32,
    inner_node_free_list_head: u32,
    inner_node_bump_index: u32,

    root_node: u32,
    pub leaf_count: u32,
}

impl SlabHeader {
    pub const LEN: usize = std::mem::size_of::<Self>();
}

pub struct Slab<'a> {
    pub header: &'a mut SlabHeader,
    pub leaf_nodes: &'a mut [LeafNode],
    pub inner_nodes: &'a mut [InnerNode],
    pub callback_infos: &'a mut [CallbackInfo],
}
#[derive(Zeroable, Clone, Copy, Pod, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct LeafNode {
    /// The key is the associated order id
    pub key: u128,
    /// The quantity of base asset associated with the underlying order
    pub base_quantity: u64,
}

impl LeafNode {
    pub const LEN: usize = std::mem::size_of::<Self>();

    /// Parse a leaf node's price
    pub fn price(&self) -> u64 {
        Self::price_from_key(self.key)
    }

    /// Get the associated order id
    pub fn order_id(&self) -> u128 {
        self.key
    }

    /// Deduce an associated price from an order_id
    pub(crate) fn price_from_key(key: u128) -> u64 {
        (key >> 64) as u64
    }
}

pub type NodeHandle = u32;

pub const INNER_FLAG: u32 = 1 << 31;
#[derive(Zeroable, Clone, Copy, Pod, Debug)]
#[repr(C)]
pub struct InnerNode {
    key: u128,
    prefix_len: u64,
    pub children: [u32; 2],
}

impl InnerNode {
    pub const LEN: usize = std::mem::size_of::<Self>();

    pub(crate) fn walk_down(&self, search_key: u128) -> (NodeHandle, bool) {
        let crit_bit_mask = (1u128 << 127) >> self.prefix_len;
        let crit_bit = (search_key & crit_bit_mask) != 0;
        (self.children[crit_bit as usize], crit_bit)
    }
}

pub enum Node {
    Leaf,
    Inner,
}

impl Node {
    pub fn from_handle(h: NodeHandle) -> Self {
        if h & INNER_FLAG == 0 {
            Self::Leaf
        } else {
            Self::Inner
        }
    }
}

impl<'slab> Slab<'slab> {
    pub fn initialize(asks_data: &mut [u8], bids_data: &mut [u8]) -> Result<()> {
        if asks_data[0] != AccountTag::Uninitialized as u8
            || bids_data[0] != AccountTag::Uninitialized as u8
        {
            return Err(anyhow::Error::msg("already initialized"));
        }
        asks_data[0] = AccountTag::Asks as u8;
        bids_data[0] = AccountTag::Bids as u8;
        Ok(())
    }

    pub fn compute_allocation_size(desired_order_capacity: usize) -> usize {
        8 + SlabHeader::LEN
            + LeafNode::LEN
            + std::mem::size_of::<CallbackInfo>()
            + (desired_order_capacity.checked_sub(1).unwrap())
                * (LeafNode::LEN + InnerNode::LEN + std::mem::size_of::<CallbackInfo>())
    }
}

impl<'a> Slab<'a> {
    pub fn from_buffer(buf: &'a mut [u8], expected_tag: AccountTag) -> Result<Self> {
        let callback_info_len = std::mem::size_of::<CallbackInfo>();
        let leaf_size = LeafNode::LEN + callback_info_len;
        let capacity = (buf.len() - SlabHeader::LEN - 8 - leaf_size) / (leaf_size + InnerNode::LEN);

        if buf[0] != expected_tag as u8 {
            return Err(anyhow::Error::msg("invalid tag"));
        }
        let (_, rem) = buf.split_at_mut(8);
        let (header, rem) = rem.split_at_mut(SlabHeader::LEN);
        let (leaves, rem) = rem.split_at_mut((capacity + 1) * LeafNode::LEN);
        let (inner_nodes, callback_infos) = rem.split_at_mut(capacity * InnerNode::LEN);
        let header = bytemuck::from_bytes_mut::<SlabHeader>(header);

        Ok(Self {
            header,
            leaf_nodes: bytemuck::cast_slice_mut::<_, LeafNode>(leaves),
            inner_nodes: bytemuck::cast_slice_mut::<_, InnerNode>(inner_nodes),
            callback_infos: bytemuck::cast_slice_mut::<_, CallbackInfo>(callback_infos),
        })
    }

    pub fn from_buffer_unchecked(buf: &'a mut [u8]) -> Result<Self> {
        let callback_info_len = std::mem::size_of::<CallbackInfo>();
        let leaf_size = LeafNode::LEN + callback_info_len;
        let capacity = (buf.len() - SlabHeader::LEN - 8 - leaf_size) / (leaf_size + InnerNode::LEN);

        let (_, rem) = buf.split_at_mut(8);
        let (header, rem) = rem.split_at_mut(SlabHeader::LEN);
        let (leaves, rem) = rem.split_at_mut((capacity + 1) * LeafNode::LEN);
        let (inner_nodes, callback_infos) = rem.split_at_mut(capacity * InnerNode::LEN);
        let header = bytemuck::from_bytes_mut::<SlabHeader>(header);

        Ok(Self {
            header,
            leaf_nodes: bytemuck::cast_slice_mut::<_, LeafNode>(leaves),
            inner_nodes: bytemuck::cast_slice_mut::<_, InnerNode>(inner_nodes),
            callback_infos: bytemuck::cast_slice_mut::<_, CallbackInfo>(callback_infos),
        })
    }
}

impl<'a> Slab<'a> {
    pub fn root(&self) -> Option<NodeHandle> {
        if self.header.leaf_count == 0 {
            None
        } else {
            Some(self.header.root_node)
        }
    }
    pub(crate) fn allocate_leaf(&mut self) -> Result<NodeHandle, IoError> {
        if self.header.leaf_free_list_len == 0 {
            if self.header.leaf_bump_index as usize >= self.leaf_nodes.len() {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }
            let key = self.header.leaf_bump_index;
            self.header.leaf_bump_index += 1;
            return Ok(key);
        }

        let key = self.header.leaf_free_list_head;
        let free_leaf = &mut self.leaf_nodes[key as usize];
        let next = free_leaf.base_quantity as u32;
        self.header.leaf_free_list_head = next;
        self.header.leaf_free_list_len -= 1;

        Ok(key)
    }

    pub(crate) fn free_leaf(&mut self, handle: NodeHandle) {
        if self.header.leaf_free_list_len != 0 {
            let next = self.header.leaf_free_list_head;
            self.leaf_nodes[handle as usize].base_quantity = next as u64;
        }

        self.header.leaf_free_list_len += 1;
        self.header.leaf_free_list_head = handle;
    }

    pub(crate) fn allocate_inner_node(&mut self) -> Result<NodeHandle, IoError> {
        if self.header.inner_node_free_list_len == 0 {
            if self.header.inner_node_bump_index as usize >= self.inner_nodes.len() {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }
            let key = self.header.inner_node_bump_index;
            self.header.inner_node_bump_index += 1;
            return Ok(!key);
        }

        let key = self.header.inner_node_free_list_head;
        let free_inner_node = &mut self.inner_nodes[key as usize];
        let next = free_inner_node.prefix_len as u32;
        self.header.inner_node_free_list_head = next;
        self.header.inner_node_free_list_len -= 1;

        Ok(!key)
    }

    pub(crate) fn free_inner_node(&mut self, handle: NodeHandle) {
        if self.header.inner_node_free_list_len != 0 {
            let next = self.header.inner_node_free_list_head;
            self.inner_nodes[(!handle) as usize].prefix_len = next as u64;
        }

        self.header.inner_node_free_list_len += 1;
        self.header.inner_node_free_list_head = !handle;
    }

    pub(crate) fn insert_leaf(
        &mut self,
        new_leaf: &LeafNode,
    ) -> Result<(NodeHandle, Option<LeafNode>)> {
        let mut root: NodeHandle = if self.header.leaf_count == 0 {
            // create a new root if none exists
            let new_leaf_handle = self
                .allocate_leaf()
                .map_err(|_| anyhow::Error::msg("out of space"))?;
            self.leaf_nodes[new_leaf_handle as usize] = *new_leaf;
            self.header.root_node = new_leaf_handle;
            self.header.leaf_count += 1;
            return Ok((new_leaf_handle, None));
        } else {
            self.header.root_node
        };
        let mut parent_node: Option<NodeHandle> = None;
        let mut previous_critbit: Option<bool> = None;
        loop {
            let shared_prefix_len = match Node::from_handle(root) {
                Node::Inner => {
                    let root_node = &self.inner_nodes[(!root) as usize];
                    let shared_prefix_len: u32 = (root_node.key ^ new_leaf.key).leading_zeros();
                    let keep_old_root = shared_prefix_len >= root_node.prefix_len as u32;
                    if keep_old_root {
                        parent_node = Some(root);
                        let r = root_node.walk_down(new_leaf.key);
                        root = r.0;
                        previous_critbit = Some(r.1);
                        continue;
                    }

                    shared_prefix_len
                }
                Node::Leaf => {
                    let root_node = &mut self.leaf_nodes[root as usize];
                    if root_node.key == new_leaf.key {
                        // clobber the existing leaf
                        let leaf_copy = *root_node;
                        *root_node = *new_leaf;
                        return Ok((root, Some(leaf_copy)));
                    }
                    let shared_prefix_len: u32 = (root_node.key ^ new_leaf.key).leading_zeros();

                    shared_prefix_len
                }
            };

            // change the root in place to represent the LCA of [new_leaf] and [root]
            let crit_bit_mask: u128 = (1u128 << 127) >> shared_prefix_len;
            let new_leaf_crit_bit = (crit_bit_mask & new_leaf.key) != 0;
            let old_root_crit_bit = !new_leaf_crit_bit;

            let new_leaf_handle = self
                .allocate_leaf()
                .map_err(|_| anyhow::Error::msg("out of space"))?;
            self.leaf_nodes[new_leaf_handle as usize] = *new_leaf;

            let new_root_node_handle = self.allocate_inner_node().unwrap();
            let new_root_node = &mut self.inner_nodes[(!new_root_node_handle) as usize];
            new_root_node.prefix_len = shared_prefix_len as u64;
            new_root_node.key = new_leaf.key;
            new_root_node.children[new_leaf_crit_bit as usize] = new_leaf_handle;
            new_root_node.children[old_root_crit_bit as usize] = root;

            if let Some(n) = parent_node {
                let node = &mut self.inner_nodes[(!n) as usize];
                node.children[previous_critbit.unwrap() as usize] = new_root_node_handle;
            } else {
                self.header.root_node = new_root_node_handle;
            }
            self.header.leaf_count += 1;
            return Ok((new_leaf_handle, None));
        }
    }

    #[inline(always)]
    pub fn get_callback_info(&self, leaf_handle: NodeHandle) -> &CallbackInfo {
        &self.callback_infos[leaf_handle as usize]
    }

    #[inline(always)]
    pub fn get_callback_info_mut(&mut self, leaf_handle: NodeHandle) -> &mut CallbackInfo {
        &mut self.callback_infos[leaf_handle as usize]
    }

    pub fn remove_by_key(&mut self, search_key: u128) -> Option<(LeafNode, &CallbackInfo)> {
        let mut grandparent_h: Option<NodeHandle> = None;
        if self.header.leaf_count == 0 {
            return None;
        }
        let mut parent_h = self.header.root_node;
        // We have to initialize the values to work around the type checker
        let mut child_h = 0;
        let mut crit_bit = false;
        let mut prev_crit_bit: Option<bool> = None;
        let mut remove_root = None;
        // let mut depth = 0;
        {
            match Node::from_handle(parent_h) {
                Node::Leaf => {
                    let leaf = &self.leaf_nodes[parent_h as usize];
                    if leaf.key == search_key {
                        remove_root = Some(*leaf);
                    }
                }
                Node::Inner => {
                    let node = self.inner_nodes[(!parent_h) as usize];
                    let (ch, cb) = node.walk_down(search_key);
                    child_h = ch;
                    crit_bit = cb;
                }
            }
        }
        if let Some(leaf_copy) = remove_root {
            self.free_leaf(parent_h);

            self.header.root_node = 0;
            self.header.leaf_count = 0;
            return Some((leaf_copy, self.get_callback_info(parent_h)));
        }
        loop {
            match Node::from_handle(child_h) {
                Node::Inner => {
                    let inner = self.inner_nodes[(!child_h) as usize];
                    let (grandchild_h, grandchild_crit_bit) = inner.walk_down(search_key);
                    grandparent_h = Some(parent_h);
                    parent_h = child_h;
                    child_h = grandchild_h;
                    prev_crit_bit = Some(crit_bit);
                    crit_bit = grandchild_crit_bit;
                    // depth += 1;
                    continue;
                }
                Node::Leaf => {
                    let leaf = &self.leaf_nodes[child_h as usize];
                    if leaf.key != search_key {
                        return None;
                    }

                    break;
                }
            }
        }

        // replace parent with its remaining child node
        // free child_h, replace *parent_h with *other_child_h, free other_child_h
        let other_child_h = self.inner_nodes[(!parent_h) as usize].children[!crit_bit as usize];

        match grandparent_h {
            Some(h) => {
                let r = &mut self.inner_nodes[(!h) as usize];
                r.children[prev_crit_bit.unwrap() as usize] = other_child_h;
            }
            None => self.header.root_node = other_child_h,
        }
        self.header.leaf_count -= 1;
        let removed_leaf = self.leaf_nodes[child_h as usize];
        self.free_leaf(child_h);
        self.free_inner_node(parent_h);
        Some((removed_leaf, self.get_callback_info(child_h)))
    }

    fn find_min_max(&self, find_max: bool) -> Option<NodeHandle> {
        if self.header.leaf_count == 0 {
            return None;
        }
        let mut root: NodeHandle = self.header.root_node;
        loop {
            match Node::from_handle(root) {
                Node::Leaf => return Some(root),
                Node::Inner => {
                    let node = self.inner_nodes[(!root) as usize];
                    root = node.children[if find_max { 1 } else { 0 }];
                }
            }
        }
    }

    /// Get the handle for the leaf of minimum key (and price)
    pub fn find_min(&self) -> Option<NodeHandle> {
        self.find_min_max(false)
    }

    /// Get the handle for the leaf of maximum key (and price)
    pub fn find_max(&self) -> Option<NodeHandle> {
        self.find_min_max(true)
    }

    /// Get a price ascending or price descending iterator over all the Slab's orders
    pub fn into_iter(self, price_ascending: bool) -> SlabIterator<'a> {
        SlabIterator {
            search_stack: if self.header.leaf_count == 0 {
                vec![]
            } else {
                vec![self.header.root_node]
            },
            slab: self,
            ascending: price_ascending,
        }
    }

    /// Get the current critbit's depth. Walks though the entire tree.
    pub fn get_depth(&self) -> usize {
        if self.header.leaf_count == 0 {
            return 0;
        }
        let mut stack = vec![(self.header.root_node, 1)];
        let mut max_depth = 0;
        while let Some((current_node, current_depth)) = stack.pop() {
            match Node::from_handle(current_node) {
                Node::Inner => {
                    let node = self.inner_nodes[(!current_node) as usize];
                    stack.push((node.children[0], current_depth + 1));
                    stack.push((node.children[1], current_depth + 1));
                }
                Node::Leaf => max_depth = std::cmp::max(current_depth, max_depth),
            }
        }
        max_depth
    }

    #[cfg(test)]
    fn check_invariants(&self) {
        // first check the live tree contents
        let mut leaf_count = 0;
        let mut inner_node_count = 0;
        fn check_rec<'a>(
            slab: &Slab<'a>,
            h: NodeHandle,
            last_prefix_len: u64,
            last_prefix: u128,
            last_critbit: bool,
            leaf_count: &mut u64,
            inner_node_count: &mut u64,
        ) {
            match Node::from_handle(h) {
                Node::Leaf => {
                    *leaf_count += 1;
                    let node = &slab.leaf_nodes[h as usize];
                    assert_eq!(
                        last_critbit,
                        (node.key & ((1u128 << 127) >> last_prefix_len)) != 0
                    );
                    let prefix_mask =
                        (((((1u128) << 127) as i128) >> last_prefix_len) as u128) << 1;
                    assert_eq!(last_prefix & prefix_mask, node.key & prefix_mask);
                }
                Node::Inner => {
                    *inner_node_count += 1;
                    let node = &slab.inner_nodes[(!h) as usize];

                    assert!(node.prefix_len > last_prefix_len);
                    assert_eq!(
                        last_critbit,
                        (node.key & ((1u128 << 127) >> last_prefix_len)) != 0
                    );
                    let prefix_mask =
                        (((((1u128) << 127) as i128) >> last_prefix_len) as u128) << 1;
                    assert_eq!(last_prefix & prefix_mask, node.key & prefix_mask);
                    check_rec(
                        slab,
                        node.children[0],
                        node.prefix_len,
                        node.key,
                        false,
                        leaf_count,
                        inner_node_count,
                    );
                    check_rec(
                        slab,
                        node.children[1],
                        node.prefix_len,
                        node.key,
                        true,
                        leaf_count,
                        inner_node_count,
                    );
                }
            }
        }
        if let Some(root) = self.root() {
            if matches!(Node::from_handle(root), Node::Inner) {
                inner_node_count += 1;
                let n = &self.inner_nodes[(!root) as usize];
                check_rec(
                    self,
                    n.children[0],
                    n.prefix_len,
                    n.key,
                    false,
                    &mut leaf_count,
                    &mut inner_node_count,
                );
                check_rec(
                    self,
                    n.children[1],
                    n.prefix_len,
                    n.key,
                    true,
                    &mut leaf_count,
                    &mut inner_node_count,
                );
            } else {
                leaf_count += 1;
            }
        }
        assert_eq!(
            inner_node_count + self.header.inner_node_free_list_len as u64,
            self.header.inner_node_bump_index as u64
        );
        assert_eq!(
            self.header.leaf_count as u64 + self.header.leaf_free_list_len as u64,
            self.header.leaf_bump_index as u64
        );
        assert_eq!(leaf_count, self.header.leaf_count as u64);
    }

    /////////////////////////////////////////
    // Misc

    pub fn find_by_key(&self, search_key: u128) -> Option<NodeHandle> {
        let mut node_handle: NodeHandle = self.root()?;
        loop {
            match Node::from_handle(node_handle) {
                Node::Leaf => {
                    let n = self.leaf_nodes[node_handle as usize];
                    if search_key == n.key {
                        return Some(node_handle);
                    } else {
                        return None;
                    }
                }
                Node::Inner => {
                    let n = self.inner_nodes[(!node_handle as usize)];
                    let common_prefix_len = (search_key ^ n.key).leading_zeros();
                    if common_prefix_len < n.prefix_len as u32 {
                        return None;
                    }
                    node_handle = n.walk_down(search_key).0;
                }
            }
        }
    }
}

impl<'queue> Slab<'queue> {
    #[cfg(test)]
    fn traverse(&self) -> Vec<(LeafNode, CallbackInfo)> {
        fn walk_rec<'a>(
            slab: &Slab<'a>,
            sub_root: NodeHandle,
            buf: &mut Vec<(LeafNode, CallbackInfo)>,
        ) {
            match Node::from_handle(sub_root) {
                Node::Leaf => {
                    let callback_info = slab.get_callback_info(sub_root);
                    buf.push((slab.leaf_nodes[sub_root as usize], *callback_info));
                }
                Node::Inner => {
                    let n = slab.inner_nodes[(!sub_root) as usize];
                    walk_rec(slab, n.children[0], buf);
                    walk_rec(slab, n.children[1], buf);
                }
            }
        }

        let mut buf = Vec::with_capacity(self.header.leaf_count as usize);
        if let Some(r) = self.root() {
            walk_rec(self, r, &mut buf);
        }
        assert_eq!(buf.len(), buf.capacity());
        buf
    }
}

pub struct SlabIterator<'a> {
    slab: Slab<'a>,
    search_stack: Vec<u32>,
    ascending: bool,
}

impl<'a> Iterator for SlabIterator<'a> {
    type Item = LeafNode;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.search_stack.pop() {
            match Node::from_handle(current) {
                Node::Inner => {
                    let n = &self.slab.inner_nodes[(!current) as usize];
                    self.search_stack.push(n.children[self.ascending as usize]);
                    self.search_stack.push(n.children[!self.ascending as usize]);
                }
                Node::Leaf => return Some(self.slab.leaf_nodes[current as usize]),
            }
        }
        None
    }
}
