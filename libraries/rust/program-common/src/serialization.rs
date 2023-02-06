use anchor_lang::AnchorSerialize;

pub trait StorageSpace {
    const SPACE: usize;
}

impl<T: Sized + AnchorSerialize> StorageSpace for T {
    const SPACE: usize = 8 + std::mem::size_of::<Self>();
}
