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
use anchor_spl::token::Token;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::sysvar::{rent::Rent, SysvarId};

use jet_margin_pool::accounts as ix_accounts;
use jet_margin_pool::instruction as ix_data;
use jet_margin_pool::program::JetMarginPool;
use jet_margin_pool::Amount;

/// Utility for creating instructions to interact with the margin
/// pools program for a specific pool.
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
    pub fn new(token_mint: Pubkey) -> Self {
        let (address, _) =
            Pubkey::find_program_address(&[token_mint.as_ref()], &JetMarginPool::id());

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
            token_program: Token::id(),
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
    /// `amount` - The amount of tokens to be deposited
    pub fn deposit(
        &self,
        depositor: Pubkey,
        source: Pubkey,
        destination: Pubkey,
        amount: u64,
    ) -> Instruction {
        let accounts = ix_accounts::Deposit {
            margin_pool: self.address,
            vault: self.vault,
            deposit_note_mint: self.deposit_note_mint,
            depositor,
            source,
            destination,
            token_program: Token::id(),
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::Deposit { amount }.data(),
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
    /// `amount` - The amount of the deposit
    pub fn withdraw(
        &self,
        depositor: Pubkey,
        source: Pubkey,
        destination: Pubkey,
        amount: Amount,
    ) -> Instruction {
        let accounts = ix_accounts::Withdraw {
            margin_pool: self.address,
            vault: self.vault,
            deposit_note_mint: self.deposit_note_mint,
            depositor,
            source,
            destination,
            token_program: Token::id(),
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::Withdraw { amount }.data(),
            accounts,
        }
    }

    /// Instruction to borrow tokens using a margin account
    ///
    /// # Params
    ///
    /// `margin_account` - The account being borrowed against
    /// `deposit_account` - The account to receive the notes for the borrowed tokens
    /// `loan_account` - The account to receive the notes representing the debt
    /// `amount` - The amount of tokens to be borrowed
    pub fn margin_borrow(
        &self,
        margin_account: Pubkey,
        deposit_account: Pubkey,
        loan_account: Pubkey,
        amount: u64,
    ) -> Instruction {
        let accounts = ix_accounts::MarginBorrow {
            margin_account,
            margin_pool: self.address,
            loan_note_mint: self.loan_note_mint,
            deposit_note_mint: self.deposit_note_mint,
            loan_account,
            deposit_account,
            token_program: Token::id(),
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::MarginBorrow { amount }.data(),
            accounts,
        }
    }

    /// Instruction to repay tokens owed by a margin account
    ///
    /// # Params
    ///
    /// `margin_account` - The account with the loan to be repaid
    /// `deposit_account` - The account with notes to repay the loan
    /// `loan_account` - The account with the loan debt to be reduced
    /// `max_amount` - The maximum amount to be repaid
    pub fn margin_repay(
        &self,
        margin_account: Pubkey,
        deposit_account: Pubkey,
        loan_account: Pubkey,
        max_amount: Amount,
    ) -> Instruction {
        let accounts = ix_accounts::MarginRepay {
            margin_account,
            margin_pool: self.address,
            loan_note_mint: self.loan_note_mint,
            deposit_note_mint: self.deposit_note_mint,
            loan_account,
            deposit_account,
            token_program: Token::id(),
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::MarginRepay { max_amount }.data(),
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

    pub fn register_loan(&self, margin_account: Pubkey, payer: Pubkey) -> (Pubkey, Instruction) {
        let loan_note_account = loan_token_account(&margin_account, &self.loan_note_mint).0;
        let position_token_metadata =
            Pubkey::find_program_address(&[self.loan_note_mint.as_ref()], &jet_metadata::ID).0;

        let accounts = ix_accounts::RegisterLoan {
            margin_account,
            position_token_metadata,
            margin_pool: self.address,
            loan_note_account,
            loan_note_mint: self.loan_note_mint,
            payer,
            token_program: Token::id(),
            system_program: System::id(),
            rent: Rent::id(),
        };

        let ix = Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::RegisterLoan {}.data(),
            accounts: accounts.to_account_metas(None),
        };

        (loan_note_account, ix)
    }

    pub fn close_loan(&self, margin_account: Pubkey, payer: Pubkey) -> Instruction {
        let loan_note_account = loan_token_account(&margin_account, &self.loan_note_mint).0;

        let accounts = ix_accounts::CloseLoan {
            margin_account,
            margin_pool: self.address,
            loan_note_account,
            loan_note_mint: self.loan_note_mint,
            beneficiary: payer,
            token_program: Token::id(),
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
            token_program: Token::id(),
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_pool::ID,
            data: ix_data::Collect.data(),
            accounts,
        }
    }
}

pub fn loan_token_account(margin_account: &Pubkey, loan_note_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[margin_account.as_ref(), loan_note_mint.as_ref()],
        &jet_margin_pool::id(),
    )
}
