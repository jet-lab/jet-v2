use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::{
    control::state::Market,
    events::TermDepositCreated,
    margin::state::MarginUser,
    orderbook::state::{CallbackFlags, CallbackInfo, SensibleOrderSummary},
    serialization,
    tickets::events::DepositRedeemed,
    FixedTermErrorCode,
};

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

pub struct InitTermDepositParams {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub tenor: u64,
    pub sequence_number: u64,
    pub auto_roll: bool,
    pub seed: Vec<u8>,
}

pub struct InitTermDepositAccounts<'a, 'info> {
    pub deposit: &'a AccountInfo<'info>,
    pub payer: &'a Signer<'info>,
    pub system_program: &'a Program<'info, System>,
}

impl<'a, 'info> InitTermDepositAccounts<'a, 'info> {
    pub fn init(
        self,
        params: InitTermDepositParams,
        info: &CallbackInfo,
        summary: &SensibleOrderSummary,
    ) -> Result<()> {
        let mut deposit = serialization::init_from_ref::<TermDeposit>(
            self.deposit,
            self.payer,
            self.system_program,
            &[
                crate::seeds::TERM_DEPOSIT,
                params.market.as_ref(),
                params.owner.as_ref(),
                &params.seed,
            ],
        )?;

        let timestamp = Clock::get()?.unix_timestamp;
        let maturation_timestamp = timestamp + params.tenor as i64;

        *deposit = TermDeposit {
            market: params.market,
            sequence_number: params.sequence_number,
            owner: params.owner,
            payer: self.payer.key(),
            matures_at: maturation_timestamp,
            principal: summary.quote_filled()?,
            amount: summary.base_filled(),
            flags: Self::flags(info),
        };
        emit!(TermDepositCreated {
            term_deposit: deposit.key(),
            authority: deposit.owner,
            payer: self.payer.key(),
            order_tag: Some(info.order_tag.as_u128()),
            sequence_number: params.sequence_number,
            market: params.market,
            maturation_timestamp,
            principal: deposit.principal,
            amount: deposit.amount,
        });
        Ok(())
    }

    fn flags(info: &CallbackInfo) -> TermDepositFlags {
        if info.flags.contains(CallbackFlags::AUTO_ROLL) {
            TermDepositFlags::AUTO_ROLL
        } else {
            TermDepositFlags::empty()
        }
    }
}

pub struct RedeemDepositAccounts<'a, 'info> {
    /// The tracking account for the deposit
    pub deposit: &'a Account<'info, TermDeposit>,

    /// The account that owns the deposit
    pub owner: &'a AccountInfo<'info>,

    /// The authority that must sign to redeem the deposit
    ///
    /// Signature check is handled in instruction logic
    pub authority: &'a AccountInfo<'info>,

    /// Receiver for the rent used to track the deposit
    pub payer: &'a AccountInfo<'info>,

    /// The token account designated to receive the assets underlying the claim
    pub token_account: &'a Account<'info, TokenAccount>,

    /// The Market responsible for the asset
    pub market: &'a AccountLoader<'info, Market>,

    /// The vault stores the tokens of the underlying asset managed by the Market
    pub underlying_token_vault: &'a Account<'info, TokenAccount>,

    /// SPL token program
    pub token_program: &'a Program<'info, Token>,
}

/// Account for the redemption of the `TermDeposit`
///
/// in the case that this function is downstream from an auto rolled lend order, there is
/// no need to withdraw funds from the vault, and `is_withdrawing` should be false
#[inline(never)]
pub fn redeem(accs: &RedeemDepositAccounts, is_withdrawing: bool) -> Result<u64> {
    let current_time = Clock::get()?.unix_timestamp;
    if current_time < accs.deposit.matures_at {
        msg!(
            "Matures at time: [{:?}]\nCurrent time: [{:?}]",
            accs.deposit.matures_at,
            current_time
        );
        return err!(FixedTermErrorCode::ImmatureTicket);
    }

    // transfer from the vault to the deposit_holder
    if is_withdrawing {
        transfer(
            CpiContext::new(
                accs.token_program.to_account_info(),
                Transfer {
                    from: accs.underlying_token_vault.to_account_info(),
                    to: accs.token_account.to_account_info(),
                    authority: accs.market.to_account_info(),
                },
            )
            .with_signer(&[&accs.market.load()?.authority_seeds()]),
            accs.deposit.amount,
        )?;
    }

    emit!(DepositRedeemed {
        deposit: accs.deposit.key(),
        deposit_holder: accs.owner.key(),
        redeemed_value: accs.deposit.amount,
        redeemed_timestamp: current_time,
    });

    Ok(accs.deposit.amount)
}

pub struct MarginRedeemDepositAccounts<'a, 'info> {
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the collateral value of assets custodied by fixed-term market
    pub ticket_collateral: &'a AccountInfo<'info>,

    /// Token mint used by the margin program to track the collateral value of assets custodied by fixed-term market
    pub ticket_collateral_mint: &'a AccountInfo<'info>,

    pub inner: &'a RedeemDepositAccounts<'a, 'info>,
}

#[inline(never)]
pub fn margin_redeem(accs: &mut MarginRedeemDepositAccounts, is_withdrawing: bool) -> Result<()> {
    let redeemed = redeem(accs.inner, is_withdrawing)?;
    accs.margin_user
        .assets
        .redeem_deposit(accs.inner.deposit.sequence_number, redeemed)?;

    anchor_spl::token::burn(
        CpiContext::new(
            accs.inner.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: accs.ticket_collateral_mint.to_account_info(),
                from: accs.ticket_collateral.to_account_info(),
                authority: accs.inner.market.to_account_info(),
            },
        )
        .with_signer(&[&accs.inner.market.load()?.authority_seeds()]),
        redeemed,
    )?;

    accs.margin_user.emit_asset_balances();

    Ok(())
}
