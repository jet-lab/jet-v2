#![allow(missing_docs)] // redundant to require here

/// Combine an item of some type with another item of the
/// same type to output a combined item of the same type
pub trait Concat {
    fn concat(self, other: Self) -> Self;
    fn concat_ref(self, other: &Self) -> Self;
}

/// joins a collection of items that implement Concat using concat method
#[macro_export]
macro_rules! cat {
    ($concattable:expr) => {
        $concattable
    };
    ($concattable:expr, $($therest:expr),+$(,)?) => {{
        use $crate::util::data::Concat;
        $concattable.concat(cat![$($therest),+])
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
            .fold(Default::default(), |acc, next| acc.concat_ref(next))
    }
}
