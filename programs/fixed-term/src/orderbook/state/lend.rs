use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

use crate::{
    control::state::Market,
    events::TermDepositCreated,
    margin::state::MarginUser,
    serialization,
    tickets::state::{TermDeposit, TermDepositFlags},
    FixedTermErrorCode,
};

use super::{CallbackFlags, CallbackInfo, SensibleOrderSummary};

pub struct LendAccounts<'a, 'info> {
    pub authority: &'a AccountInfo<'info>,
    pub market: &'a AccountLoader<'info, Market>,
    pub ticket_mint: &'a Account<'info, Mint>,
    pub ticket_settlement: &'a AccountInfo<'info>,
    pub lender_tokens: &'a Account<'info, TokenAccount>,
    pub underlying_token_vault: &'a Account<'info, TokenAccount>,
    pub payer: &'a Signer<'info>,
    pub token_program: &'a Program<'info, Token>,
    pub system_program: &'a Program<'info, System>,
}

pub struct MarginLendAccounts<'a, 'info> {
    pub margin_user: Box<Account<'info, MarginUser>>,
    pub ticket_collateral: &'a AccountInfo<'info>,
    pub ticket_collateral_mint: &'a AccountInfo<'info>,
    pub inner: &'a LendAccounts<'a, 'info>,
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

pub struct TicketMintAccounts<'a, 'info> {
    pub market: &'a AccountLoader<'info, Market>,
    pub ticket_mint: &'a Account<'info, Mint>,
    pub ticket_settlement: &'a AccountInfo<'info>,
    pub token_program: &'a Program<'info, Token>,
}

/// How to account for lent tokens
pub enum Issuance<'a, 'info> {
    TermDeposit(InitTermDepositAccounts<'a, 'info>, InitTermDepositParams),
    Tickets(TicketMintAccounts<'a, 'info>),
}

impl<'a, 'info> Issuance<'a, 'info> {
    fn staked(self) -> Result<(InitTermDepositAccounts<'a, 'info>, InitTermDepositParams)> {
        match self {
            Self::TermDeposit(accounts, params) => Ok((accounts, params)),
            _ => err!(FixedTermErrorCode::WrongIssuance),
        }
    }

    fn tickets(self) -> Result<TicketMintAccounts<'a, 'info>> {
        match self {
            Self::Tickets(accounts) => Ok(accounts),
            _ => err!(FixedTermErrorCode::WrongIssuance),
        }
    }
}

pub fn lend(
    accounts: &LendAccounts,
    deposit_params: Option<InitTermDepositParams>,
    order_callback: &CallbackInfo,
    order_summary: &SensibleOrderSummary,
    requires_payment: bool,
) -> Result<u64> {
    let issuance = if order_callback.flags.contains(CallbackFlags::AUTO_STAKE) {
        Issuance::TermDeposit(
            InitTermDepositAccounts {
                deposit: accounts.ticket_settlement,
                payer: accounts.payer,
                system_program: accounts.system_program,
            },
            deposit_params.ok_or_else(|| FixedTermErrorCode::MissingTermDepositParameters)?,
        )
    } else {
        Issuance::Tickets(TicketMintAccounts {
            market: accounts.market,
            ticket_mint: accounts.ticket_mint,
            ticket_settlement: accounts.ticket_settlement,
            token_program: accounts.token_program,
        })
    };

    if requires_payment {
        // take all underlying that has been lent plus what may be lent later
        anchor_spl::token::transfer(
            anchor_lang::prelude::CpiContext::new(
                accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: accounts.lender_tokens.to_account_info(),
                    to: accounts.underlying_token_vault.to_account_info(),
                    authority: accounts.authority.to_account_info(),
                },
            ),
            order_summary.quote_combined()?,
        )?;
    }

    issue_lend(order_callback, order_summary, issuance)
}

/// Account for tokens that have been lent by an order placement
///
/// Returns the amount of staked tickets
fn issue_lend(
    order_callback: &CallbackInfo,
    order_summary: &SensibleOrderSummary,
    issuance: Issuance,
) -> Result<u64> {
    let staked = if order_summary.base_filled() > 0 {
        if order_callback.flags.contains(CallbackFlags::AUTO_STAKE) {
            let (accounts, params) = issuance.staked()?;
            create_term_deposit(params, accounts, order_summary, order_callback)?;
            order_summary.base_filled()
        } else {
            // no auto_stake: issue free tickets to the user for immediate fill
            let accounts = issuance.tickets()?;
            issue_tickets(accounts, order_summary.base_filled())?;
            0
        }
    } else {
        0
    };

    Ok(staked)
}

pub fn margin_lend(
    accounts: &mut MarginLendAccounts,
    deposit_params: Option<InitTermDepositParams>,
    order_callback: &CallbackInfo,
    order_summary: &SensibleOrderSummary,
    requires_payment: bool,
) -> Result<()> {
    let staked = lend(
        accounts.inner,
        deposit_params,
        order_callback,
        order_summary,
        requires_payment,
    )?;

    if staked > 0 {
        accounts.margin_user.assets.new_deposit(staked)?;
    }

    mint_to(
        CpiContext::new(
            accounts.inner.token_program.to_account_info(),
            MintTo {
                mint: accounts.ticket_collateral_mint.to_account_info(),
                to: accounts.ticket_collateral.to_account_info(),
                authority: accounts.inner.market.to_account_info(),
            },
        )
        .with_signer(&[&accounts.inner.market.load()?.authority_seeds()]),
        staked + order_summary.quote_posted()?,
    )?;

    Ok(())
}

fn create_term_deposit(
    params: InitTermDepositParams,
    accounts: InitTermDepositAccounts,
    order_summary: &SensibleOrderSummary,
    order_callback: &CallbackInfo,
) -> Result<()> {
    let mut deposit = serialization::init_from_ref::<TermDeposit>(
        accounts.deposit,
        accounts.payer,
        accounts.system_program,
        &[
            crate::seeds::TERM_DEPOSIT,
            params.market.as_ref(),
            params.owner.as_ref(),
            &params.seed,
        ],
    )?;

    let timestamp = Clock::get()?.unix_timestamp;
    let maturation_timestamp = timestamp + params.tenor as i64;

    let auto_roll = if order_callback.flags.contains(CallbackFlags::AUTO_ROLL) {
        TermDepositFlags::AUTO_ROLL
    } else {
        TermDepositFlags::empty()
    };

    *deposit = TermDeposit {
        market: params.market,
        sequence_number: params.sequence_number,
        owner: params.owner,
        payer: accounts.payer.key(),
        matures_at: maturation_timestamp,
        principal: order_summary.quote_filled()?,
        amount: order_summary.base_filled(),
        flags: TermDepositFlags::default() | auto_roll,
    };
    emit!(TermDepositCreated {
        term_deposit: deposit.key(),
        authority: deposit.owner,
        payer: accounts.payer.key(),
        order_tag: Some(order_callback.order_tag.as_u128()),
        sequence_number: params.sequence_number,
        market: params.market,
        maturation_timestamp,
        principal: deposit.principal,
        amount: deposit.amount,
    });
    Ok(())
}

fn issue_tickets(accounts: TicketMintAccounts, amount: u64) -> Result<()> {
    mint_to(
        CpiContext::new(
            accounts.token_program.to_account_info(),
            MintTo {
                mint: accounts.ticket_mint.to_account_info(),
                to: accounts.ticket_settlement.clone(),
                authority: accounts.market.to_account_info(),
            },
        )
        .with_signer(&[&accounts.market.load()?.authority_seeds()]),
        amount,
    )?;

    Ok(())
}
