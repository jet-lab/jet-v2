#![allow(missing_docs)] // redundant to require here

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

/// joins a collection of items that implement Concat using concat method
#[macro_export]
macro_rules! cat {
    ($concattable:expr) => {
        $concattable
    };
    ($concattable:expr, $($therest:expr),+$(,)?) => {{
        use $crate::util::data::Concat;
        $concattable.cat(cat![$($therest),+])
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
