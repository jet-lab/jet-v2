// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anchor_lang::prelude::{Id, System, ToAccountMetas};
use anchor_lang::InstructionData;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::sysvar::{rent::Rent, SysvarId};

use jet_margin_pool::instruction as ix_data;
use jet_margin_pool::program::JetMarginPool;
use jet_margin_pool::{accounts as ix_accounts, TokenChange};

pub use jet_margin_pool::ID as MARGIN_POOL_PROGRAM;

use crate::margin::MarginConfigIxBuilder;

/// Utility for creating instructions to interact with the margin
/// pools program for a specific pool.
#[derive(Clone)]
pub struct MarginPoolIxBuilder {
    /// The address of the mint for tokens stored in the pool
    pub token_mint: Pubkey,

    /// The address of the margin pool
    pub address: Pubkey,

    /// The address of the account holding the tokens in the pool
    pub vault: Pubkey,

    /// The address of the mint for deposit notes, which represent user
    /// deposit in the pool
    pub deposit_note_mint: Pubkey,

    /// The address of the mint for loan notes, which represent user borrows
    /// from the pool
    pub loan_note_mint: Pubkey,
}

impl MarginPoolIxBuilder {
    /// Create a new builder for an SPL token mint by deriving pool addresses
    ///
    /// # Params
    ///
    /// `token_mint` - The token mint which whose tokens the pool stores
    pub fn new(token_mint: Pubkey) -> Self {
        let address = derive_margin_pool(&Default::default(), &token_mint);

        let (vault, _) = Pubkey::find_program_address(
            &[address.as_ref(), b"vault".as_ref()],
            &JetMarginPool::id(),
        );

        let (deposit_note_mint, _) = Pubkey::find_program_address(
            &[address.as_ref(), b"deposit-notes".as_ref()],
            &JetMarginPool::id(),
        );

        let (loan_note_mint, _) = Pubkey::find_program_address(
            &[address.as_ref(), b"loan-notes".as_ref()],
            &JetMarginPool::id(),
        );

        Self {
            token_mint,
            address,
            vault,
            deposit_note_mint,
            loan_note_mint,
        }
    }

    /// Instruction to create the pool with given parameters
    ///
    /// # Params
    ///
    /// `payer` - The address paying for the rent
    pub fn create(&self, payer: Pubkey, fee_destination: Pubkey) -> Instruction {
        let authority = match cfg!(feature = "devnet") {
            true => payer,
            false => jet_margin_pool::authority::ID,
        };
        let accounts = ix_accounts::CreatePool {
            authority,
            token_mint: self.token_mint,
            margin_pool: self.address,
            deposit_note_mint: self.deposit_note_mint,
            loan_note_mint: self.loan_note_mint,
            vault: self.vault,
            payer,
            token_program: spl_token::ID,
            system_program: System::id(),
            rent: Rent::id(),
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::CreatePool { fee_destination }.data(),
            accounts,
        }
    }

    /// Instruction to deposit tokens into the pool in exchange for deposit notes
    ///
    /// # Params
    ///
    /// `depositor` - The authority for the source tokens
    /// `source` - The token account that has the tokens to be deposited
    /// `destination` - The token account to send notes representing the deposit
    /// `change` - The type of token change being made. See [TokenChange].
    pub fn deposit(
        &self,
        depositor: Pubkey,
        source: Pubkey,
        destination: Pubkey,
        change: TokenChange,
    ) -> Instruction {
        let accounts = ix_accounts::Deposit {
            margin_pool: self.address,
            vault: self.vault,
            deposit_note_mint: self.deposit_note_mint,
            depositor,
            source,
            destination,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        let TokenChange { kind, tokens } = change;
        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::Deposit {
                change_kind: kind,
                amount: tokens,
            }
            .data(),
            accounts,
        }
    }

    /// Instruction to withdraw tokens from the pool in exchange for deposit notes
    ///
    /// # Params
    ///
    /// `depositor` - The authority for the deposit notes
    /// `source` - The token account that has the deposit notes to be exchanged
    /// `destination` - The token account to send the withdrawn deposit
    /// `change` - The amount of the deposit
    pub fn withdraw(
        &self,
        depositor: Pubkey,
        source: Pubkey,
        destination: Pubkey,
        change: TokenChange,
    ) -> Instruction {
        let accounts = ix_accounts::Withdraw {
            margin_pool: self.address,
            vault: self.vault,
            deposit_note_mint: self.deposit_note_mint,
            depositor,
            source,
            destination,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        let TokenChange { kind, tokens } = change;
        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::Withdraw {
                change_kind: kind,
                amount: tokens,
            }
            .data(),
            accounts,
        }
    }

    /// Instruction to borrow tokens using a margin account
    ///
    /// # Params
    ///
    /// `margin_account` - The account being borrowed against
    /// `deposit_account` - The account to receive the notes for the borrowed tokens
    /// `amount` - The amount of tokens to be borrowed
    pub fn margin_borrow(
        &self,
        margin_account: Pubkey,
        deposit_account: Pubkey,
        change: TokenChange,
    ) -> Instruction {
        let accounts = ix_accounts::MarginBorrow {
            margin_account,
            margin_pool: self.address,
            loan_note_mint: self.loan_note_mint,
            deposit_note_mint: self.deposit_note_mint,
            loan_account: derive_loan_account(&margin_account, &self.loan_note_mint),
            deposit_account,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        let TokenChange { kind, tokens } = change;
        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::MarginBorrow {
                change_kind: kind,
                amount: tokens,
            }
            .data(),
            accounts,
        }
    }

    /// Instruction to borrow tokens using a margin account
    ///
    /// # Params
    ///
    /// `margin_account` - The account being borrowed against
    /// `destination` - The account to receive the borrowed tokens
    /// `amount` - The amount of tokens to be borrowed
    pub fn margin_borrow_v2(
        &self,
        margin_account: Pubkey,
        destination: Pubkey,
        amount: u64,
    ) -> Instruction {
        let accounts = ix_accounts::MarginBorrowV2 {
            margin_account,
            margin_pool: self.address,
            loan_note_mint: self.loan_note_mint,
            vault: self.vault,
            loan_account: derive_loan_account(&margin_account, &self.loan_note_mint),
            destination,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::MarginBorrowV2 { amount }.data(),
            accounts,
        }
    }

    /// Instruction to repay tokens owed by a margin account
    ///
    /// # Params
    ///
    /// `margin_account` - The account with the loan to be repaid
    /// `deposit_account` - The account with notes to repay the loan
    /// `amount` - The amount to be repaid
    pub fn margin_repay(
        &self,
        margin_account: Pubkey,
        deposit_account: Pubkey,
        change: TokenChange,
    ) -> Instruction {
        let accounts = ix_accounts::MarginRepay {
            margin_account,
            margin_pool: self.address,
            loan_note_mint: self.loan_note_mint,
            deposit_note_mint: self.deposit_note_mint,
            loan_account: derive_loan_account(&margin_account, &self.loan_note_mint),
            deposit_account,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        let TokenChange { kind, tokens } = change;
        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::MarginRepay {
                change_kind: kind,
                amount: tokens,
            }
            .data(),
            accounts,
        }
    }

    /// Instruction to repay tokens owed by a margin account using a token account
    ///
    /// # Params
    ///
    /// `margin_account` - The account with the loan to be repaid
    /// `repayment_source_authority` - The authority for the repayment source tokens
    /// `repayment_source_account` - The token account to use for repayment
    /// `loan_account` - The account with the loan debt to be reduced
    /// `amount` - The amount to be repaid
    pub fn repay(
        &self,
        repayment_source_authority: Pubkey,
        repayment_source_account: Pubkey,
        loan_account: Pubkey,
        change: TokenChange,
    ) -> Instruction {
        let accounts = ix_accounts::Repay {
            margin_pool: self.address,
            loan_note_mint: self.loan_note_mint,
            vault: self.vault,
            loan_account,
            repayment_token_account: repayment_source_account,
            repayment_account_authority: repayment_source_authority,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        let TokenChange { kind, tokens } = change;
        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::Repay {
                change_kind: kind,
                amount: tokens,
            }
            .data(),
            accounts,
        }
    }

    /// Instruction to refresh the position on a margin account
    ///
    /// # Params
    ///
    /// `margin_account` - The margin account with the deposit to be withdrawn
    /// `oracle` - The oracle account for this pool
    pub fn margin_refresh_position(&self, margin_account: Pubkey, oracle: Pubkey) -> Instruction {
        let accounts = ix_accounts::MarginRefreshPosition {
            margin_account,
            margin_pool: self.address,
            token_price_oracle: oracle,
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::MarginRefreshPosition {}.data(),
            accounts,
        }
    }

    /// Instruction to register a loan position with a margin pool.
    pub fn register_loan(
        &self,
        margin_account: Pubkey,
        payer: Pubkey,
        airspace: Pubkey,
    ) -> Instruction {
        let loan_note_account = derive_loan_account(&margin_account, &self.loan_note_mint);
        let loan_token_config = MarginConfigIxBuilder::new(airspace, payer, None)
            .derive_token_config(&self.loan_note_mint);

        let accounts = ix_accounts::RegisterLoan {
            margin_account,
            loan_token_config,
            margin_pool: self.address,
            loan_note_account,
            loan_note_mint: self.loan_note_mint,
            payer,
            token_program: spl_token::ID,
            system_program: System::id(),
            rent: Rent::id(),
        };

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::RegisterLoan {}.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Instruction to close a loan account in a margin pool
    pub fn close_loan(&self, margin_account: Pubkey, payer: Pubkey) -> Instruction {
        let loan_note_account = derive_loan_account(&margin_account, &self.loan_note_mint);

        let accounts = ix_accounts::CloseLoan {
            margin_account,
            margin_pool: self.address,
            loan_note_account,
            loan_note_mint: self.loan_note_mint,
            beneficiary: payer,
            token_program: spl_token::ID,
        };

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::CloseLoan {}.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Instruction to collect interest and fees
    pub fn collect(&self, fee_destination: Pubkey) -> Instruction {
        let accounts = ix_accounts::Collect {
            margin_pool: self.address,
            vault: self.vault,
            fee_destination,
            deposit_note_mint: self.deposit_note_mint,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::Collect.data(),
            accounts,
        }
    }

    /// Instruction to transfer a loan between margin accounts
    pub fn admin_transfer_loan(
        &self,
        source_margin_account: &Pubkey,
        target_margin_account: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let accounts = ix_accounts::AdminTransferLoan {
            authority: jet_program_common::GOVERNOR_ID,
            margin_pool: self.address,
            source_loan_account: derive_loan_account(source_margin_account, &self.loan_note_mint),
            target_loan_account: derive_loan_account(target_margin_account, &self.loan_note_mint),
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::AdminTransferLoan { amount }.data(),
            accounts,
        }
    }
}

/// Find a loan token account for a margin account and margin pool's loan note mint
pub fn derive_loan_account(margin_account: &Pubkey, loan_note_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[margin_account.as_ref(), loan_note_mint.as_ref()],
        &jet_margin_pool::id(),
    )
    .0
}

/// Derive the address for a margin pool
pub fn derive_margin_pool(_airspace: &Pubkey, token_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[token_mint.as_ref()], &jet_margin_pool::ID).0
}
