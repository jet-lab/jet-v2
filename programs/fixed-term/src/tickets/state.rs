use anchor_lang::prelude::*;

/// A representation of an interest earning deposit, which can be redeemed after reaching maturity
#[account]
#[derive(Debug)]
pub struct TermDeposit {
    /// The owner of the redeemable tokens
    ///
    /// This is usually a user's margin account, unless the deposit was created directly
    /// with this program.
    pub owner: Pubkey,

    /// The relevant market for this deposit
    pub market: Pubkey,

    /// Which account recieves the rent when this PDA is destructed
    pub payer: Pubkey,

    /// The sequence number for this deposit, which serves as unique identifier for a
    /// particular user's deposits.
    pub sequence_number: u64,

    /// The timestamp at which this deposit has matured, and can be redeemed
    pub matures_at: i64,

    /// The number of tokens that can be reedeemed at maturity
    pub amount: u64,

    /// The number tokens originally provided to create this deposit
    ///
    /// This is only accurate when using the auto-stake feature, which saves the original
    /// token amount provided in the loan order.
    pub principal: u64,

    /// Any boolean flags for this data type compressed to a single byte
    pub flags: TermDepositFlags,
}

bitflags! {
    #[derive(Default, AnchorSerialize, AnchorDeserialize)]
    pub struct TermDepositFlags: u8 {
        /// This term loan has already been marked as due.
        const AUTO_ROLL = 0b00000001;
    }
}
