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

use std::{convert::TryFrom, num::NonZeroU64};

use anchor_openbook::serum_dex::{
    instruction::SelfTradeBehavior,
    matching::{OrderType, Side},
};
use anchor_spl::dex as anchor_openbook;

use crate::*;

#[derive(Accounts)]
pub struct OpenbookSwapInfo<'info> {
    /// CHECK:
    #[account(mut)]
    pub market: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub base_vault: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub quote_vault: AccountInfo<'info>,

    /// CHECK:
    pub vault_signer: AccountInfo<'info>,

    /// The address of the swap program
    pub dex_program: Program<'info, anchor_openbook::Dex>,

    pub rent: Sysvar<'info, Rent>,
}

impl<'info> OpenbookSwapInfo<'info> {
    #[inline(never)]
    pub fn swap(
        &self,
        source: &AccountInfo<'info>,
        target: &AccountInfo<'info>,
        authority: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        amount_in: u64,
        _minimum_amount_out: u64,
    ) -> Result<()> {
        let (base_lot_size, base_mint) = {
            let market =
                anchor_openbook::serum_dex::state::Market::load(&self.market, self.dex_program.key)
                    .unwrap();
            let base_mint = Pubkey::try_from(bytemuck::cast_slice(&{ market.coin_mint })).unwrap();
            (market.coin_lot_size, base_mint)
        };

        // Determine order side
        let source_mint = token::accessor::mint(source)?;
        let (side, base_wallet, quote_wallet, limit_price, max_base, max_quote) =
            if source_mint == base_mint {
                let limit_price = 1;
                let max_base = amount_in.checked_div(base_lot_size).unwrap();
                let max_quote = u64::MAX;
                (Side::Ask, source, target, limit_price, max_base, max_quote)
            } else {
                let limit_price = u64::MAX;
                let max_base = u64::MAX;
                let max_quote = amount_in;
                (Side::Bid, target, source, limit_price, max_base, max_quote)
            };

        let swap_context = CpiContext::new(
            self.dex_program.to_account_info(),
            anchor_openbook::NewOrderV3 {
                market: self.market.to_account_info(),
                open_orders: self.open_orders.to_account_info(),
                request_queue: self.request_queue.to_account_info(),
                event_queue: self.event_queue.to_account_info(),
                market_bids: self.market_bids.to_account_info(),
                market_asks: self.market_asks.to_account_info(),
                order_payer_token_account: source.to_account_info(),
                open_orders_authority: authority.to_account_info(),
                coin_vault: self.base_vault.to_account_info(),
                pc_vault: self.quote_vault.to_account_info(),
                token_program: token_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
        );

        anchor_openbook::new_order_v3(
            swap_context,
            side,
            NonZeroU64::new(limit_price).unwrap(),
            NonZeroU64::new(max_base).unwrap(),
            NonZeroU64::new(max_quote).unwrap(),
            SelfTradeBehavior::AbortTransaction,
            OrderType::ImmediateOrCancel,
            0,
            u16::MAX,
        )?;

        let settle_ctx = CpiContext::new(
            self.dex_program.to_account_info(),
            anchor_openbook::SettleFunds {
                market: self.market.to_account_info(),
                open_orders: self.open_orders.to_account_info(),
                open_orders_authority: authority.to_account_info(),
                coin_vault: self.base_vault.to_account_info(),
                pc_vault: self.quote_vault.to_account_info(),
                token_program: token_program.to_account_info(),
                coin_wallet: base_wallet.to_account_info(),
                pc_wallet: quote_wallet.to_account_info(),
                vault_signer: self.vault_signer.to_account_info(),
            },
        );
        anchor_openbook::settle_funds(settle_ctx)?;

        Ok(())
    }
}
