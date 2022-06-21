pub mod spl_token_swap_v2 {
    pub use spl_token_swap_v2::*;

    crate::program!(Spl2, "SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8");
}

pub mod orca_swap_v1 {
    pub use spl_token_swap_3613cea3c::*;

    crate::program!(OrcaV1, "DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1");
}

pub mod orca_swap_v2 {
    pub use spl_token_swap_813aa3::*;

    #[cfg(not(feature = "devnet"))]
    crate::program!(OrcaV2, "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP");

    #[cfg(feature = "devnet")]
    crate::program!(OrcaV2, "3xQ8SWv2GaFXXpHZNqkXsdxq5DZciHBz6ZFoPPfbFd7U");
}
