pub mod actions;
pub mod context;
pub mod environment;
pub mod fixed_term;
pub mod load;
pub mod margin;
pub mod openbook;
pub mod pricing;
pub mod runtime;
pub mod saber_swap;
pub mod setup_helper;
pub mod spl_swap;
pub mod test_user;
pub mod tokens;
pub mod util;

pub fn test_default<T: TestDefault>() -> T {
    TestDefault::test_default()
}

/// Sane defaults that can be used for fields you don't care about.
pub trait TestDefault {
    fn test_default() -> Self;
}
