use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::{
    control::state::Market,
    events::TermDepositCreated,
    margin::state::MarginUser,
    orderbook::state::CallbackFlags,
    serialization::{self, AnchorAccount, Mut},
    tickets::events::DepositRedeemed,
    FixedTermErrorCode,
};

/// A representation of an interest earning deposit, which can be redeemed after reaching maturity
#[account]
#[derive(Debug)]
pub struct TermDeposit {
    /// The owner of the redeemable tokens
    ///
    /// If this deposit was created by a Margin account, this is the `MarginAccount`.
    /// Else, this is the Pubkey of the lender's signing account
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

impl TermDeposit {
    pub fn seeds<'a>(market: &'a [u8], owner: &'a [u8], seed: &'a [u8]) -> [&'a [u8]; 4] {
        [crate::seeds::TERM_DEPOSIT, market, owner, seed]
    }
}

pub struct InitTermDepositAccounts<'a, 'info> {
    pub deposit: &'a AccountInfo<'info>,
    pub payer: &'a Signer<'info>,
    pub system_program: &'a Program<'info, System>,
}

pub struct TermDepositWriter {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub payer: Pubkey,
    pub order_tag: u128,
    pub tenor: u64,
    pub sequence_number: u64,
    pub amount: u64,
    pub principal: u64,
    pub flags: TermDepositFlags,
    pub seed: Vec<u8>,
}

impl TermDepositWriter {
    /// Initializes the `TermDeposit` anchor account and writes the `self` parameters into it
    pub fn init_and_write(self, init_accs: InitTermDepositAccounts) -> Result<()> {
        let deposit = &mut serialization::init_from_ref::<TermDeposit>(
            init_accs.deposit,
            init_accs.payer,
            init_accs.system_program,
            &TermDeposit::seeds(self.market.as_ref(), self.owner.as_ref(), &self.seed),
        )?;
        self.write(deposit)
    }
    /// writes itself into an already initialized `TermDeposit` struct
    pub fn write(self, deposit: &mut AnchorAccount<TermDeposit, Mut>) -> Result<()> {
        let maturation_timestamp = Clock::get()?.unix_timestamp + self.tenor as i64;
        **deposit = TermDeposit {
            market: self.market,
            sequence_number: self.sequence_number,
            owner: self.owner,
            payer: self.payer,
            matures_at: maturation_timestamp,
            principal: self.principal,
            amount: self.amount,
            flags: self.flags,
        };
        emit!(TermDepositCreated {
            term_deposit: deposit.key(),
            authority: deposit.owner,
            payer: self.payer,
            order_tag: Some(self.order_tag),
            sequence_number: self.sequence_number,
            market: self.market,
            maturation_timestamp,
            principal: deposit.principal,
            amount: deposit.amount,
        });
        Ok(())
    }
}

bitflags! {
    #[derive(Default, AnchorSerialize, AnchorDeserialize)]
    pub struct TermDepositFlags: u8 {
        /// Can this term deposit be redeemed and replaced on the orderbook
        const AUTO_ROLL = 1;

        /// Is this term deposit custodied by the margin account
        const MARGIN    = 1 << 1;
    }
}

impl From<CallbackFlags> for TermDepositFlags {
    fn from(info: CallbackFlags) -> Self {
        let mut flags = Self::empty();
        if info.contains(CallbackFlags::AUTO_ROLL) {
            flags |= Self::AUTO_ROLL;
        }
        if info.contains(CallbackFlags::MARGIN) {
            flags |= Self::MARGIN;
        }
        flags
    }
}

pub struct RedeemDepositAccounts<'a, 'info> {
    /// The tracking account for the deposit
    pub deposit: &'a Account<'info, TermDeposit>,

    /// The account that owns the deposit
    pub owner: &'a AccountInfo<'info>,

    /// Receiver for the rent used to track the deposit
    pub payer: &'a AccountInfo<'info>,

    /// The token account designated to receive the assets underlying the claim
    pub token_account: &'a AccountInfo<'info>,

    /// The Market responsible for the asset
    pub market: &'a AccountLoader<'info, Market>,

    /// The vault stores the tokens of the underlying asset managed by the Market
    pub underlying_token_vault: &'a Account<'info, TokenAccount>,

    /// SPL token program
    pub token_program: &'a Program<'info, Token>,
}

impl<'a, 'info> RedeemDepositAccounts<'a, 'info> {
    /// Account for the redemption of the `TermDeposit`
    ///
    /// in the case that this function is downstream from an auto rolled lend order, there is
    /// no need to withdraw funds from the vault, and `is_withdrawing` should be false
    pub fn redeem(&self, is_withdrawing: bool) -> Result<u64> {
        let current_time = Clock::get()?.unix_timestamp;
        if current_time < self.deposit.matures_at {
            msg!(
                "Matures at time: [{:?}]\nCurrent time: [{:?}]",
                self.deposit.matures_at,
                current_time
            );
            return err!(FixedTermErrorCode::ImmatureTicket);
        }

        // transfer from the vault to the deposit_holder
        if is_withdrawing {
            transfer(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    Transfer {
                        from: self.underlying_token_vault.to_account_info(),
                        to: self.token_account.to_account_info(),
                        authority: self.market.to_account_info(),
                    },
                )
                .with_signer(&[&self.market.load()?.authority_seeds()]),
                self.deposit.amount,
            )?;
        }

        emit!(DepositRedeemed {
            deposit: self.deposit.key(),
            deposit_holder: self.owner.key(),
            redeemed_value: self.deposit.amount,
            redeemed_timestamp: current_time,
        });

        Ok(self.deposit.amount)
    }
}

pub struct MarginRedeemDepositAccounts<'a, 'info> {
    pub margin_user: &'a mut Account<'info, MarginUser>,

    /// Token account used by the margin program to track the collateral value of assets custodied by fixed-term market
    pub ticket_collateral: &'a AccountInfo<'info>,

    /// Token mint used by the margin program to track the collateral value of assets custodied by fixed-term market
    pub ticket_collateral_mint: &'a AccountInfo<'info>,

    pub inner: &'a RedeemDepositAccounts<'a, 'info>,
}

impl<'a, 'info> MarginRedeemDepositAccounts<'a, 'info> {
    /// Run TermDeposit redemption and margin accounting logic
    pub fn margin_redeem(&mut self, is_withdrawing: bool) -> Result<()> {
        let redeemed = self.inner.redeem(is_withdrawing)?;

        self.margin_user
            .assets
            .redeem_deposit(self.inner.deposit.sequence_number, redeemed)?;

        anchor_spl::token::burn(
            CpiContext::new(
                self.inner.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: self.ticket_collateral_mint.to_account_info(),
                    from: self.ticket_collateral.to_account_info(),
                    authority: self.inner.market.to_account_info(),
                },
            )
            .with_signer(&[&self.inner.market.load()?.authority_seeds()]),
            redeemed,
        )?;

        self.margin_user.emit_asset_balances();

        Ok(())
    }
}
