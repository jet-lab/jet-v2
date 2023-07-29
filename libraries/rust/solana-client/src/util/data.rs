use std::collections::HashSet;
use std::hash::Hash;

/// Combine an item of some type with another item of the
/// same type to output a combined item of the same type
pub trait Concat {
    fn cat(self, other: Self) -> Self;
    fn cat_ref(self, other: &Self) -> Self;
}

impl<T: Clone> Concat for Vec<T> {
    fn cat(mut self, mut other: Self) -> Self {
        self.append(&mut other);
        self
    }

    fn cat_ref(mut self, other: &Self) -> Self {
        let mut other: Vec<T> = other.to_vec();
        self.append(&mut other);
        self
    }
}

/// Add an item to a collection and return the collection
pub trait With {
    type Inner;
    fn with(self, other: Self::Inner) -> Self;
}

impl<T> With for Vec<T> {
    type Inner = T;

    fn with(mut self, other: Self::Inner) -> Self {
        self.push(other);
        self
    }
}

pub trait DeepReverse {
    fn deep_reverse(self) -> Self;
}

impl<T: DeepReverse> DeepReverse for Vec<T> {
    fn deep_reverse(mut self) -> Self {
        self.reverse();
        self.into_iter().map(DeepReverse::deep_reverse).collect()
    }
}

/// joins a collection of items that implement Concat using concat method
#[macro_export]
macro_rules! cat {
    ($concattable:expr) => {
        $concattable
    };
    ($concattable:expr, $($therest:expr),+$(,)?) => {{
        use $crate::util::data::Concat;
        $concattable.cat($crate::cat![$($therest),+])
    }};
}

/// Combine a collection of items into a single item
pub trait Join {
    type Output;
    fn ijoin(self) -> Self::Output;
}

impl<'a, T: Default + Concat + 'a, I: IntoIterator<Item = &'a T>> Join for I {
    type Output = T;

    fn ijoin(self) -> Self::Output {
        self.into_iter()
            .fold(Default::default(), |acc, next| acc.cat_ref(next))
    }
}

#[test]
fn cat_vec() {
    let one = vec![1, 2, 3];
    let two = vec![4, 5, 6];
    assert_eq!(one.clone().cat_ref(&two), one.clone().cat(two.clone()));
    assert_eq!(one.cat_ref(&two), [1, 2, 3, 4, 5, 6]);
}

/// returns a vec including only the items from v that had the indices in
/// to_keep.
pub(crate) fn retain<T>(v: impl IntoIterator<Item = T>, to_keep: Vec<usize>) -> Vec<T> {
    if to_keep.is_empty() {
        return vec![];
    }
    let mut ret = Vec::with_capacity(to_keep.len());
    let mut to_keep = to_keep.into_iter();
    let mut next_keep = to_keep.next().unwrap();
    for (index, item) in v.into_iter().enumerate() {
        if next_keep == index {
            ret.push(item);
            match to_keep.next() {
                Some(x) => next_keep = x,
                None => return ret,
            }
        }
    }
    ret
}

/// returns a vec including only the items from v that had the indices in
/// to_keep.
pub(crate) fn retain_cloned<'a, T: Clone + 'a>(
    v: impl IntoIterator<Item = &'a T>,
    mut to_keep: Vec<usize>,
) -> Vec<T> {
    if to_keep.is_empty() {
        return vec![];
    }
    to_keep.sort();
    let mut ret = Vec::with_capacity(to_keep.len());
    let mut to_keep = to_keep.into_iter();
    let mut next_keep = to_keep.next().unwrap();
    for (index, item) in v.into_iter().enumerate() {
        if next_keep == index {
            ret.push(T::clone(item));
            match to_keep.next() {
                Some(x) => next_keep = x,
                None => return ret,
            }
        }
    }
    ret
}

/// this is faster than using HashSet::intersection for non-sets because that
/// requires you to first create a set, and then it executes this logic.
pub(crate) fn intersect<'a, T: Hash + Eq + Clone + 'a>(
    set: &HashSet<T>,
    not_set: impl IntoIterator<Item = &'a T>,
) -> HashSet<T> {
    not_set
        .into_iter()
        .filter(|t| set.contains(t))
        .map(Clone::clone)
        .collect()
}
