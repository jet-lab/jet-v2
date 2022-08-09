use solana_sdk::signature::Keypair;

pub mod asynchronous;
pub mod context;
pub mod load;
pub mod margin;
pub mod orchestrator;
pub mod setup_helper;
pub mod swap;
pub mod tokens;
pub mod transaction_builder;

pub use asynchronous::*;
pub use transaction_builder::*;

pub fn clone(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}

pub fn clone_vec(vec: &[Keypair]) -> Vec<Keypair> {
    vec.iter().map(clone).collect()
}

pub trait Concat {
    fn concat(self, other: Self) -> Self;
    fn concat_ref(self, other: &Self) -> Self;
}

macro_rules! cat {
    ($concattable:expr, $($therest:expr),+$(,)?) => {
        $concattable.concat(cat!($($therest)+))
    };
    ($concattable:expr) => {
        $concattable
    };
}
pub(crate) use cat;

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
