import {
  AssociatedToken,
  FixedTermMarket,
  MarginAccount,
  MarketAndconfig,
  Pool,
  redeem,
  repay,
  settle,
  TokenAmount
} from '@jet-lab/margin';
import { Deposit, Loan } from '@jet-lab/store';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import BN from 'bn.js';
import { AnchorProvider } from '@project-serum/anchor';
import { Dispatch, SetStateAction } from 'react';
import { JetMarginPools } from '@state/pools/pools';

export const submitRepay = async (
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  amount: BN,
  termLoans: Loan[],
  pools: Record<string, Pool>,
  markets: FixedTermMarket[],
  market: MarketAndconfig,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solscan' | 'solanaExplorer' | 'solanaBeach'
) => {
  let tx = 'failed_before_tx';
  try {
    tx = await repay({
      provider,
      marginAccount,
      amount,
      termLoans,
      pools,
      markets,
      market
    });
    notify(
      'Repay Successful',
      `Your debt has been successfully repaid`,
      'success',
      getExplorerUrl(tx, cluster, blockExplorer)
    );
  } catch (e: any) {
    notify(
      'Repay Failed',
      `There was an issue repaying your debt, please try again.`,
      'error',
      getExplorerUrl(e.signature, cluster, blockExplorer)
    );
    throw e;
  }
};

export const getOwedTokens = async (
  market: MarketAndconfig,
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  setOwedTokens: Dispatch<SetStateAction<TokenAmount>>
) => {
  const mint = market.token.mint;
  const pda = AssociatedToken.derive(mint, marginAccount.address);
  try {
    const exists = await provider.connection.getAccountInfo(pda);
    if (exists) {
      const balance = await provider.connection.getTokenAccountBalance(pda);
      setOwedTokens(new TokenAmount(new BN(balance.value.amount || 0), balance.value.decimals));
    }
  } catch (e) {
    console.log(e);
  }
};

export const settleNow = async (
  marginAccount: MarginAccount,
  markets: MarketAndconfig[],
  selectedMarket: MarketAndconfig,
  provider: AnchorProvider,
  setOwedTokens: Dispatch<SetStateAction<TokenAmount>>,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solscan' | 'solanaExplorer' | 'solanaBeach',
  pools: JetMarginPools,
  amount: TokenAmount
) => {
  const token = selectedMarket.token;
  if (!marginAccount || !token) return;
  let tx = 'failed_before_tx';
  try {
    tx = await settle({
      markets,
      selectedMarket,
      marginAccount,
      provider,
      pools: pools.tokenPools,
      amount: amount.lamports
    });
    notify(
      'Settle Successful',
      `Your assets have been sent to your margin account`,
      'success',
      getExplorerUrl(tx, cluster, blockExplorer)
    );
    setOwedTokens(new TokenAmount(new BN(0), token.decimals));
  } catch (e: any) {
    notify(
      'Settle Failed',
      `There was an issue settling your funds, please try again.`,
      'error',
      getExplorerUrl(e.signature, cluster, blockExplorer)
    );
  }
};

export const redeemDeposits = async (
  market: MarketAndconfig,
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  deposits: Deposit[],
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solanaExplorer' | 'solscan' | 'solanaBeach',
  pools: Record<string, Pool>,
  markets: FixedTermMarket[]
) => {
  try {
    await redeem({
      market,
      marginAccount,
      provider,
      pools,
      markets,
      deposits
    });
    notify('Deposit Redeemed', 'Your deposit was successfully redeem', 'success');
  } catch (e: any) {
    notify(
      'Deposit redemption failed',
      'There was an error redeeming your deposit',
      'error',
      getExplorerUrl(e.signature, cluster, blockExplorer)
    );
    throw e;
  }
};