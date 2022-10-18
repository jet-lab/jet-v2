// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use std::cmp::Ordering;

use bytemuck::{Contiguous, Pod, Zeroable};
use thiserror::Error;

const NULL_INDEX: u16 = u16::MAX;
const MAGIC: [u8; 4] = *b"tree";

#[derive(Error, Debug)]
pub enum TreeError {
    /// Attempting to use an unitialized tree
    #[error("the tree was not initialized")]
    Uninitialized,

    /// Attempting to initialize a tree in a buffer that was previously initialized
    #[error("the tree is already initialized")]
    Initialized,

    /// The storage for the stree is full
    #[error("there is no space left to insert a new entry")]
    Full,

    /// Trying to insert a key that already is in the tree
    #[error("the key already exists in the tree")]
    KeyExists,
}

/// A tree that can use a byte buffer as storage, suitable for using in on-chain account data
///
/// This is implemented as a red-black tree. The entire state for the tree is stored in the
/// underlying buffer provided, and tree operations do not require any memory allocations. The
/// capacity for this tree is determined by the total number of nodes that can fit within the
/// underlying buffer, and thus extending the capacity can be done by extending the length of
/// the byte buffer.
pub struct Tree<'a, K, V>
where
    K: Pod,
    V: Pod,
{
    header: &'a mut TreeHeader,
    nodes: &'a mut [TreeNode<K, V>],
}

impl<'a, K, V> Tree<'a, K, V>
where
    K: Pod + Ord,
    V: Pod,
{
    /// The size of one node, which stores a key-value entry.
    pub const NODE_SIZE: usize = std::mem::size_of::<TreeNode<K, V>>();

    /// Initialize a new tree using a given buffer as storage
    ///
    /// The provided buffer *must* be zero-initialized.
    pub fn new(buffer: &'a mut [u8]) -> Result<Self, TreeError> {
        if &buffer[..4] != &[0u8; 4] {
            return Err(TreeError::Initialized);
        }

        let tree = Self::load_unchecked(buffer);
        tree.header.magic = MAGIC;
        tree.header.root = NULL_INDEX;
        tree.header.free_index = 0;

        Ok(tree)
    }

    /// Load the tree from a previously intialized buffer
    pub fn load(buffer: &'a mut [u8]) -> Result<Self, TreeError> {
        if &buffer[..4] != &MAGIC {
            return Err(TreeError::Uninitialized);
        }

        Ok(Self::load_unchecked(buffer))
    }

    fn load_unchecked(buffer: &'a mut [u8]) -> Self {
        let (header_buffer, nodes_buffer) = buffer.split_at_mut(std::mem::size_of::<TreeHeader>());
        let (_, nodes_buffer, _) = bytemuck::pod_align_to_mut(nodes_buffer);

        Self {
            header: bytemuck::from_bytes_mut(header_buffer),
            nodes: nodes_buffer,
        }
    }

    /// Get the maximum number of entries that can be stored
    pub fn capacity(&self) -> usize {
        self.nodes.len()
    }

    /// Retrieve a value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        let (x, _) = self.find_near(key);

        if x == NULL_INDEX {
            return None;
        }

        Some(self.value(x))
    }

    /// Iterate over all entries
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        Iter {
            tree: self,
            begin: self.min(self.header.root),
            end: self.max(self.header.root),
        }
    }

    /// Insert a new entry
    pub fn insert(&mut self, key: K, value: V) -> Result<(), TreeError> {
        let node_index = match self.allocate() {
            None => return Err(TreeError::Full),
            Some(index) => index,
        };

        let (exact, nearest) = self.find_near(&key);

        let allocated = &mut self.nodes[node_index as usize];
        allocated.child_left = NULL_INDEX;
        allocated.child_right = NULL_INDEX;
        allocated.color = Color::Red.into_integer();
        allocated.key = key;
        allocated.value = value;

        if nearest == NULL_INDEX {
            self.header.root = node_index;
            allocated.parent = NULL_INDEX;
            allocated.color = Color::Black.into_integer();
            return Ok(());
        }

        if exact != NULL_INDEX {
            return Err(TreeError::KeyExists);
        }

        println!("inserting {node_index}");

        if key < *self.key(nearest) {
            self.set_left_child(nearest, node_index);
        } else {
            self.set_right_child(nearest, node_index);
        }

        self.rebalance_after_insert(node_index);
        Ok(())
    }

    /// Remove an existing entry
    pub fn remove(&mut self, key: &K) {
        let z = match self.find_near(key) {
            (z, _) if z == NULL_INDEX => return,
            (z, _) => z,
        };

        let y = if self.left_child(z) == NULL_INDEX || self.right_child(z) == NULL_INDEX {
            z
        } else {
            self.next(z)
        };
        let mut x = if self.left_child(y) != NULL_INDEX {
            self.left_child(y)
        } else {
            self.right_child(y)
        };
        let mut w = NULL_INDEX;

        if y != self.header.root {
            if self.is_left_child(y) {
                self.set_left_child(self.parent(y), x);
                w = self.right_child(self.parent(y));
            } else {
                self.set_right_child(self.parent(y), x);
                w = self.left_child(self.parent(y));
            }
        } else {
            self.set_root(x);
        }

        let color_y = self.color(y);

        if y != z {
            if self.header.root == z {
                self.set_root(y);
            } else {
                if self.is_left_child(z) {
                    self.set_left_child(self.parent(z), y);
                } else {
                    self.set_right_child(self.parent(z), y);
                }
            }

            self.set_left_child(y, self.left_child(z));
            self.set_right_child(y, self.right_child(z));

            self.set_color(y, self.color(z));
        }

        if color_y == Color::Black && self.header.root != NULL_INDEX {
            if x != NULL_INDEX {
                self.set_color(x, Color::Black);
            } else {
                loop {
                    if !self.is_left_child(w) {
                        if self.color(w) == Color::Red {
                            self.set_color(w, Color::Black);
                            self.set_color(self.parent(w), Color::Red);

                            self.rotate_left(self.parent(w));

                            w = self.right_child(self.left_child(w));
                        }

                        if (self.left_child(w) == NULL_INDEX
                            || self.color(self.left_child(w)) == Color::Black)
                            && (self.right_child(w) == NULL_INDEX
                                || self.color(self.right_child(w)) == Color::Black)
                        {
                            self.set_color(w, Color::Red);
                            x = self.parent(w);

                            if x == self.header.root || self.color(x) == Color::Red {
                                self.set_color(x, Color::Black);
                                break;
                            }

                            if self.is_left_child(x) {
                                w = self.right_child(self.parent(x));
                            } else {
                                w = self.left_child(self.parent(x));
                            }
                        } else {
                            if self.right_child(w) == NULL_INDEX
                                || self.color(self.right_child(w)) == Color::Black
                            {
                                self.set_color(self.left_child(w), Color::Black);
                                self.set_color(w, Color::Red);

                                self.rotate_right(w);

                                w = self.parent(w);
                            }

                            self.set_color(w, self.color(self.parent(w)));
                            self.set_color(self.parent(w), Color::Black);
                            self.set_color(self.right_child(w), Color::Black);

                            self.rotate_left(self.parent(w));
                            break;
                        }
                    } else {
                        if self.color(w) == Color::Red {
                            self.set_color(w, Color::Black);
                            self.set_color(self.parent(w), Color::Red);
                            self.rotate_right(self.parent(w));

                            w = self.left_child(self.right_child(w));
                        }

                        if (self.left_child(w) == NULL_INDEX
                            || self.color(self.left_child(w)) == Color::Black)
                            && (self.right_child(w) == NULL_INDEX
                                || self.color(self.right_child(w)) == Color::Black)
                        {
                            self.set_color(w, Color::Red);
                            x = self.parent(w);

                            if self.color(x) == Color::Red || x == self.header.root {
                                self.set_color(x, Color::Black);
                                break;
                            }

                            if self.is_left_child(x) {
                                w = self.right_child(self.parent(x));
                            } else {
                                w = self.left_child(self.parent(x));
                            }
                        } else {
                            if self.left_child(w) == NULL_INDEX
                                || self.color(self.left_child(w)) == Color::Black
                            {
                                self.set_color(self.right_child(w), Color::Black);
                                self.set_color(w, Color::Red);
                                self.rotate_left(w);

                                w = self.parent(w);
                            }

                            self.set_color(w, self.color(self.parent(w)));
                            self.set_color(self.parent(w), Color::Black);
                            self.set_color(self.left_child(w), Color::Black);
                            self.rotate_right(self.parent(w));
                            break;
                        }
                    }
                }
            }
        }

        self.free(z);
    }

    fn rebalance_after_insert(&mut self, mut x: u16) {
        while self.header.root != x && self.color(self.parent(x)) == Color::Red {
            let gp_x = self.parent(self.parent(x));

            if self.is_left_child(self.parent(x)) {
                let y = self.right_child(gp_x);

                if y != NULL_INDEX && self.color(y) == Color::Red {
                    x = self.parent(x);
                    self.set_color(x, Color::Black);
                    x = self.parent(x);

                    if x == self.header.root {
                        self.set_color(x, Color::Black);
                    } else {
                        self.set_color(x, Color::Red);
                    }

                    self.set_color(y, Color::Black);
                } else {
                    if !self.is_left_child(x) {
                        x = self.parent(x);
                        self.rotate_left(x);
                    }

                    x = self.parent(x);
                    self.set_color(x, Color::Black);
                    x = self.parent(x);
                    self.set_color(x, Color::Red);

                    self.rotate_right(x);
                }
            } else {
                let y = self.left_child(gp_x);

                if y != NULL_INDEX && self.color(y) == Color::Red {
                    x = self.parent(x);
                    self.set_color(x, Color::Black);
                    x = self.parent(x);

                    if x == self.header.root {
                        self.set_color(x, Color::Black);
                    } else {
                        self.set_color(x, Color::Red);
                    }

                    self.set_color(y, Color::Black);
                } else {
                    if self.is_left_child(x) {
                        x = self.parent(x);
                        self.rotate_right(x);
                    }

                    x = self.parent(x);
                    self.set_color(x, Color::Black);
                    x = self.parent(x);
                    self.set_color(x, Color::Red);

                    self.rotate_left(x)
                }
            }
        }

        self.set_color(self.header.root, Color::Black);
    }

    fn rotate_left(&mut self, x: u16) {
        let y = self.right_child(x);

        self.set_right_child(x, self.left_child(y));

        if self.parent(x) != NULL_INDEX {
            if self.is_left_child(x) {
                self.set_left_child(self.parent(x), y);
            } else {
                self.set_right_child(self.parent(x), y);
            }
        } else {
            self.set_root(y);
        }

        println!("y is {y}");
        self.set_left_child(y, x);
    }

    fn rotate_right(&mut self, x: u16) {
        let y = self.left_child(x);

        self.set_left_child(x, self.right_child(y));

        if self.parent(x) != NULL_INDEX {
            if self.is_left_child(x) {
                self.set_left_child(self.parent(x), y);
            } else {
                self.set_right_child(self.parent(x), y);
            }
        } else {
            self.set_root(y);
        }

        self.set_right_child(y, x);
    }

    fn find_near(&self, key: &K) -> (u16, u16) {
        let mut x = self.header.root;

        println!("root is {x}");

        if x != NULL_INDEX {
            loop {
                let order = key.cmp(self.key(x));

                match order {
                    Ordering::Less => {
                        if self.left_child(x) != NULL_INDEX {
                            x = self.left_child(x);
                        } else {
                            return (NULL_INDEX, x);
                        }
                    }

                    Ordering::Greater => {
                        if self.right_child(x) != NULL_INDEX {
                            x = self.right_child(x);
                        } else {
                            return (NULL_INDEX, x);
                        }
                    }

                    _ => return (x, x),
                }
            }
        }

        (NULL_INDEX, NULL_INDEX)
    }

    fn next(&self, mut x: u16) -> u16 {
        if self.right_child(x) != NULL_INDEX {
            return self.min(self.right_child(x));
        }

        while !self.is_left_child(x) {
            x = self.parent(x);
        }

        self.parent(x)
    }

    fn is_left_child(&self, x: u16) -> bool {
        let node_x = &self.nodes[x as usize];
        let parent_x = &self.nodes[node_x.parent as usize];

        parent_x.child_left == x
    }

    fn max(&self, mut index: u16) -> u16 {
        loop {
            let node = &self.nodes[index as usize];

            if node.child_right == NULL_INDEX {
                break index;
            }

            index = node.child_right;
        }
    }

    fn min(&self, mut index: u16) -> u16 {
        loop {
            let node = &self.nodes[index as usize];

            if node.child_left == NULL_INDEX {
                break index;
            }

            index = node.child_left;
        }
    }

    fn allocate(&mut self) -> Option<u16> {
        if self.header.free_index == NULL_INDEX {
            return None;
        }

        let free_index = self.header.free_index;
        let free_node = &mut self.nodes[free_index as usize];

        if free_node.color() == Color::None {
            self.header.free_index += 1;

            if self.header.free_index as usize >= self.nodes.as_ref().len() {
                self.header.free_index = NULL_INDEX;
            }
        } else {
            self.header.free_index = free_node.child_left;
        }

        Some(free_index)
    }

    fn free(&mut self, index: u16) {
        let node = &mut self.nodes[index as usize];

        node.color = Color::Unfilled.into_integer();
        node.child_left = self.header.free_index;
        node.child_right = NULL_INDEX;
        node.key = K::zeroed();
        node.value = V::zeroed();

        self.header.free_index = index;
    }

    fn key(&self, index: u16) -> &K {
        &self.nodes[index as usize].key
    }

    fn value(&self, index: u16) -> &V {
        &self.nodes[index as usize].value
    }

    fn parent(&self, index: u16) -> u16 {
        self.nodes[index as usize].parent
    }

    fn right_child(&self, index: u16) -> u16 {
        self.nodes[index as usize].child_right
    }

    fn left_child(&self, index: u16) -> u16 {
        self.nodes[index as usize].child_left
    }

    fn color(&self, index: u16) -> Color {
        self.nodes[index as usize].color()
    }

    fn set_left_child(&mut self, index: u16, child: u16) {
        self.nodes[index as usize].child_left = child;

        if child != NULL_INDEX {
            self.nodes[child as usize].parent = index;
            println!("{child} is left of {index}");
        }
    }

    fn set_right_child(&mut self, index: u16, child: u16) {
        self.nodes[index as usize].child_right = child;

        if child != NULL_INDEX {
            self.nodes[child as usize].parent = index;
            println!("{child} is right of {index}");
        }
    }

    fn set_color(&mut self, index: u16, color: Color) {
        self.nodes[index as usize].color = color.into_integer()
    }

    fn set_parent(&mut self, child: u16, parent: u16) {
        self.nodes[child as usize].parent = parent
    }

    fn set_root(&mut self, index: u16) {
        self.header.root = index;
        self.set_parent(index, NULL_INDEX);
    }
}

#[derive(Contiguous, Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
enum Color {
    None = 0,
    Unfilled,
    Black,
    Red,
}

#[derive(Zeroable, Clone, Copy)]
#[repr(C)]
struct TreeNode<K, V>
where
    K: Pod,
    V: Pod,
{
    color: u8,
    parent: u16,
    child_left: u16,
    child_right: u16,
    _reserved: [u8; 1],

    key: K,
    value: V,
}

impl<K: Pod, V: Pod> std::fmt::Debug for TreeNode<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeNode")
            .field("color", &self.color())
            .field("parent", &self.parent)
            .field("left", &self.child_left)
            .field("right", &self.child_right)
            .finish_non_exhaustive()
    }
}

unsafe impl<K: Pod, V: Pod> Pod for TreeNode<K, V> {}

impl<K: Pod, V: Pod> TreeNode<K, V> {
    fn color(&self) -> Color {
        Color::from_integer(self.color).unwrap()
    }
}

#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct TreeHeader {
    magic: [u8; 4],
    root: u16,
    free_index: u16,
    _reserved: [u8; 2],
}

pub struct Iter<'a, K: Pod, V: Pod> {
    tree: &'a Tree<'a, K, V>,
    begin: u16,
    end: u16,
}

impl<'a, K: Pod + Ord, V: Pod> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let begin = self.begin;

        if begin == NULL_INDEX {
            return None;
        }

        if begin == self.end {
            self.begin = NULL_INDEX;
            self.end = NULL_INDEX;
        } else {
            self.begin = self.tree.next(begin);
        }

        Some((self.tree.key(begin), self.tree.value(begin)))
    }
}
