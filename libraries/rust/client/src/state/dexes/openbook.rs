use std::sync::Arc;

use anchor_lang::ToAccountMetas;
use anchor_spl::dex::serum_dex::state::{gen_vault_signer_key, MarketState};
use solana_sdk::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, rent::Rent,
    sysvar::SysvarId,
};

use jet_environment::client_config::DexInfo;
use jet_instructions::margin_swap::SwapAccounts;
use jet_margin_swap::{
    accounts::OpenbookSwapInfo, seeds::OPENBOOK_OPEN_ORDERS, SwapRouteIdentifier,
};
use jet_program_common::{programs::OPENBOOK, CONTROL_AUTHORITY};
use jet_solana_client::rpc::SolanaRpcExtra;
use spl_associated_token_account::get_associated_token_address;

use crate::{bail, state::AccountStates, ClientResult};

use super::DexState;

pub async fn load_openbook_markets(
    states: &AccountStates,
    markets: &[DexInfo],
) -> ClientResult<()> {
    let market_addrs = markets.iter().map(|w| w.address).collect::<Vec<_>>();

    log::debug!("loading openbook markets: {market_addrs:?}");

    let market_states = states.network.get_accounts_all(&market_addrs).await?;

    for (index, account_state) in market_states.into_iter().enumerate() {
        let address = market_addrs[index];
        let Some(mut account_state) = account_state else {
            continue;
        };

        let account_info = AccountInfo::from((&address, &mut account_state));
        let Ok(state ) = MarketState::load(&account_info, &OPENBOOK) else {
            log::warn!("failed to load openbook market {address}");
            continue;
        };

        let state = match OpenBookMarket::from_market_state(address, OPENBOOK, &state) {
            Ok(state) => state,
            Err(e) => {
                log::warn!("failed to load openbook market {address}: {e:?}");
                continue;
            }
        };

        let swap_accounts = Arc::new(state);
        let dex_state = DexState {
            program: OPENBOOK,
            token_a: state.base_mint,
            token_b: state.quote_mint,
            swap_a_to_b_accounts: swap_accounts.clone(),
            swap_b_to_a_accounts: swap_accounts.clone(),
        };

        states.set(&address, dex_state);
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct OpenBookMarket {
    /// The market address
    pub market: Pubkey,
    /// Base (coin) mint
    pub base_mint: Pubkey,
    /// Quote (price currency) mint
    pub quote_mint: Pubkey,
    ///
    pub request_queue: Pubkey,
    ///
    pub bids: Pubkey,
    ///
    pub asks: Pubkey,
    ///
    pub event_queue: Pubkey,
    ///
    pub base_vault: Pubkey,
    ///
    pub quote_vault: Pubkey,
    ///
    pub vault_signer: Pubkey,
    ///
    pub program: Pubkey,
    ///
    pub base_lot_size: u64,
    ///
    pub quote_lot_size: u64,
    /// Base decimals for price conversions
    pub base_mint_decimals: u8,
    /// Quote decimals for price conversions
    pub quote_mint_decimals: u8,
}

impl OpenBookMarket {
    fn from_market_state(
        market_address: Pubkey,
        program: Pubkey,
        market: &MarketState,
    ) -> ClientResult<Self> {
        let Ok(vault_signer) =
            gen_vault_signer_key(market.vault_signer_nonce, &market_address, &program) else {
                bail!("could not generate vault signing address for market {market_address}");
            };

        Ok(Self {
            program,
            market: market_address,
            base_mint: bytemuck::cast(market.coin_mint),
            quote_mint: bytemuck::cast(market.pc_mint),
            request_queue: bytemuck::cast(market.req_q),
            bids: bytemuck::cast(market.bids),
            asks: bytemuck::cast(market.asks),
            event_queue: bytemuck::cast(market.event_q),
            base_vault: bytemuck::cast(market.coin_vault),
            quote_vault: bytemuck::cast(market.pc_vault),
            vault_signer,
            base_lot_size: market.coin_lot_size,
            quote_lot_size: market.pc_lot_size,
            base_mint_decimals: 0,
            quote_mint_decimals: 0,
        })
    }
}

impl SwapAccounts for OpenBookMarket {
    fn to_account_meta(&self, authority: Pubkey) -> Vec<AccountMeta> {
        let (open_orders, _) = Pubkey::find_program_address(
            &[
                OPENBOOK_OPEN_ORDERS,
                authority.as_ref(),
                self.market.as_ref(),
            ],
            &jet_margin_swap::id(),
        );

        let referrer = get_associated_token_address(&CONTROL_AUTHORITY, &self.quote_mint);

        OpenbookSwapInfo {
            market: self.market,
            /// This relies on a deterministic open orders account
            open_orders,
            request_queue: self.request_queue,
            event_queue: self.event_queue,
            market_bids: self.bids,
            market_asks: self.asks,
            base_vault: self.base_vault,
            quote_vault: self.quote_vault,
            quote_mint: self.quote_mint,
            vault_signer: self.vault_signer,
            referrer_account: referrer,
            dex_program: self.program,
            rent: Rent::id(),
        }
        .to_account_metas(None)
    }

    fn pool_tokens(&self) -> (Pubkey, Pubkey) {
        (self.base_mint, self.quote_mint)
    }

    fn route_type(&self) -> SwapRouteIdentifier {
        SwapRouteIdentifier::OpenBook
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
}
