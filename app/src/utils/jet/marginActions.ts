import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
// import axios from 'axios';
import { TransactionInstruction } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { chunks, MarginAccount, Pool, PoolTokenChange, SPLSwapPool, TokenAmount, TokenFaucet } from '@jet-lab/margin';
import { MainConfig } from '@state/config/marginConfig';
import { Pools, CurrentPool } from '@state/pools/pools';
import { WalletTokens } from '@state/user/walletTokens';
import { CurrentAccount, CurrentAccountAddress, FavoriteAccounts } from '@state/user/accounts';
import { Dictionary } from '@state/settings/localization/localization';
import { TokenInputAmount, ActionRefresh } from '@state/actions/actions';
import { useProvider } from './provider';
import { NOTIFICATION_DURATION } from '../notify';
import { message } from 'antd';
import { Cluster } from '@state/settings/settings';

export enum ActionResponse {
  Success = 'SUCCESS',
  Failed = 'FAILED',
  Cancelled = 'CANCELLED'
}
export function useMarginActions() {
  // const cluster = useRecoilValue(Cluster);
  const config = useRecoilValue(MainConfig);
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const { programs, provider } = useProvider();
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const wallet = useWallet();
  const walletTokens = useRecoilValue(WalletTokens);
  const currentAccount = useRecoilValue(CurrentAccount);
  const setCurrentAccountAddress = useSetRecoilState(CurrentAccountAddress);
  const [favoriteAccounts, setFavoriteAccounts] = useRecoilState(FavoriteAccounts);
  const accountPoolPosition = currentPool?.symbol && currentAccount?.poolPositions[currentPool.symbol];
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const setActionRefresh = useSetRecoilState(ActionRefresh);

  // Refresh to trigger new data fetching after a timeout
  async function actionRefresh() {
    setActionRefresh(Date.now());
  }

  // If on devnet, user can airdrop themself tokens
  async function airdrop(pool: Pool): Promise<[string, string | undefined, ActionResponse]> {
    if (!config) {
      throw new Error('No Config');
    }
    if (!programs) {
      throw new Error('No Programs');
    }
    if (!wallet.publicKey) {
      throw new Error('No Public Key');
    }
    if (cluster === 'mainnet-beta') {
      throw new Error('Cannot airdrop on mainnet');
    }

    // Airdrop 10 tokens by default
    let amount = TokenAmount.tokens(10, pool.decimals);

    if (pool.symbol == 'USDC') {
      // provide larger amounts for USDC like
      amount = TokenAmount.tokens(100_000, pool.decimals);
    } else if (pool.symbol == 'SOL') {
      if (cluster == 'localnet') {
        amount = TokenAmount.tokens(100, pool.decimals);
      } else {
        amount = TokenAmount.tokens(1, pool.decimals);
      }
    }
    const token = config.tokens[pool.symbol] ? config.tokens[pool.symbol] : config.tokens[pool.name];
    try {
      const txId = await TokenFaucet.airdrop(provider, cluster, amount.lamports, token, wallet.publicKey);
      await actionRefresh();
      return [amount.uiTokens, txId, ActionResponse.Success];
    } catch (err) {
      console.error(err);
      return ['0', undefined, ActionResponse.Failed];
    }
  }

  // Create Account
  async function createAccount(accountName?: string): Promise<[string | undefined, ActionResponse]> {
    if (!programs || !pools || !currentPool || !walletTokens || !wallet.publicKey) {
      console.error('Pools not loaded');
      throw new Error();
    }

    try {
      // Create new account
      const seed = await MarginAccount.getUnusedAccountSeed({ programs, provider, owner: wallet.publicKey });
      const newMarginAccount = new MarginAccount(
        programs,
        provider,
        wallet.publicKey,
        seed,
        pools.tokenPools,
        walletTokens
      );

      await newMarginAccount.createAccount();

      // TODO add account names back
      // if (accountName) {
      //   axios
      //     .put(``, {
      //       alias: accountName,
      //       network: cluster,
      //       publicKey: newMarginAccount.address.toString()
      //     })
      //     .catch(err => err);
      // }

      // Update favorite accounts and set UI to new account
      const favoriteAccountsClone = { ...favoriteAccounts };
      const favoriteWalletAccounts = favoriteAccountsClone[wallet.publicKey.toString()] ?? [];
      const newWalletFavorites: string[] = [...favoriteWalletAccounts];
      newWalletFavorites.push(newMarginAccount.address.toString());
      favoriteAccountsClone[wallet.publicKey.toString()] = newWalletFavorites;
      setFavoriteAccounts(favoriteAccountsClone);
      setCurrentAccountAddress(newMarginAccount.address.toString());

      await actionRefresh();
      return [undefined, ActionResponse.Success];
    } catch (err: any) {
      console.error(err);
      if (err.toString().includes('User rejected') || err.toString().includes('Failed to sign')) {
        return [undefined, ActionResponse.Cancelled];
      } else {
        return [undefined, ActionResponse.Failed];
      }
    }
  }

  // Deposit
  async function deposit(): Promise<[string | undefined, ActionResponse]> {
    if (!pools || !currentPool || !walletTokens || !currentAccount || !accountPoolPosition) {
      console.error('Accounts and/or pools not loaded');
      throw new Error();
    }

    try {
      const txId = await currentPool.deposit({
        marginAccount: currentAccount,
        change: PoolTokenChange.setTo(accountPoolPosition.depositBalance.add(tokenInputAmount)),
        source: walletTokens.map[currentPool.symbol].address
      });
      await actionRefresh();
      return [txId, ActionResponse.Success];
    } catch (err: any) {
      console.error(err);
      if (err.toString().includes('User rejected') || err.toString().includes('Failed to sign')) {
        return [undefined, ActionResponse.Cancelled];
      } else {
        return [undefined, ActionResponse.Failed];
      }
    }
  }

  // Withdraw
  async function withdraw(): Promise<[string | undefined, ActionResponse]> {
    if (!pools?.tokenPools || !currentPool || !walletTokens || !currentAccount || !accountPoolPosition) {
      console.error('Accounts and/or pools not loaded');
      throw new Error();
    }

    const destination = walletTokens.map[currentPool.symbol];
    const change = tokenInputAmount.eq(accountPoolPosition.maxTradeAmounts.withdraw)
      ? PoolTokenChange.setTo(0)
      : PoolTokenChange.setTo(accountPoolPosition.depositBalance.sub(tokenInputAmount));
    try {
      const txId = await currentPool.withdraw({
        marginAccount: currentAccount,
        pools: Object.values(pools.tokenPools),
        destination: destination.address,
        change
      });
      await actionRefresh();
      return [txId, ActionResponse.Success];
    } catch (err: any) {
      console.error(err);
      if (err.toString().includes('User rejected') || err.toString().includes('Failed to sign')) {
        return [undefined, ActionResponse.Cancelled];
      } else {
        return [undefined, ActionResponse.Failed];
      }
    }
  }

  // Borrow
  async function borrow(): Promise<[string | undefined, ActionResponse]> {
    if (!pools || !currentPool || !walletTokens || !currentAccount || !accountPoolPosition) {
      console.error('Accounts and/or pools not loaded');
      throw new Error();
    }

    try {
      const txId = await currentPool.marginBorrow({
        marginAccount: currentAccount,
        pools: Object.values(pools.tokenPools),
        change: PoolTokenChange.shiftBy(tokenInputAmount)
      });
      await actionRefresh();
      return [txId, ActionResponse.Success];
    } catch (err: any) {
      console.error(err);
      if (err.toString().includes('User rejected') || err.toString().includes('Failed to sign')) {
        return [undefined, ActionResponse.Cancelled];
      } else {
        return [undefined, ActionResponse.Failed];
      }
    }
  }

  // Repay
  async function repay(accountRepay: boolean): Promise<[string | undefined, ActionResponse]> {
    if (!pools || !currentPool || !walletTokens || !currentAccount || !accountPoolPosition) {
      console.error('Accounts and/or pools not loaded');
      throw new Error();
    }

    const closeLoan = tokenInputAmount.gte(accountPoolPosition.loanBalance);
    const change = closeLoan ? PoolTokenChange.setTo(0) : PoolTokenChange.shiftBy(tokenInputAmount);

    try {
      const txId = await currentPool.marginRepay({
        marginAccount: currentAccount,
        source: accountRepay ? undefined : walletTokens.map[currentPool.symbol].address,
        pools: Object.values(pools.tokenPools),
        change,
        closeLoan
      });
      await actionRefresh();
      return [txId, ActionResponse.Success];
    } catch (err: any) {
      console.error(err);
      if (err.toString().includes('User rejected') || err.toString().includes('Failed to sign')) {
        return [undefined, ActionResponse.Cancelled];
      } else {
        return [undefined, ActionResponse.Failed];
      }
    }
  }

  // Swap
  async function splTokenSwap(
    inputToken: Pool,
    outputToken: Pool,
    swapPool: SPLSwapPool,
    swapAmount: TokenAmount,
    minAmountOut: TokenAmount,
    repayWithOutput: boolean
  ): Promise<[string | undefined, ActionResponse | undefined]> {
    if (!pools || !inputToken || !outputToken || !currentAccount) {
      console.error('Input/output tokens or current account undefined');
      throw new Error();
    }

    try {
      const txId = await inputToken.splTokenSwap({
        marginAccount: currentAccount,
        pools: Object.values(pools.tokenPools),
        outputToken,
        swapPool,
        swapAmount,
        minAmountOut,
        repayWithOutput
      });
      await actionRefresh();
      if (txId === 'Setup check failed') {
        return [undefined, ActionResponse.Failed];
      }
      return [txId, ActionResponse.Success];
    } catch (err: any) {
      console.error(err);
      if (err.toString().includes('User rejected') || err.toString().includes('Failed to sign')) {
        return [undefined, ActionResponse.Cancelled];
      } else if (err.toString().includes('"Custom":16')) {
        message.warning(dictionary.actions.swap.warningMessages.maxSlippageExceeded, NOTIFICATION_DURATION);
        return [undefined, undefined];
      } else {
        return [undefined, ActionResponse.Failed];
      }
    }
  }

  // Transfer
  async function transfer(
    fromAccount: MarginAccount,
    toAccount: MarginAccount
  ): Promise<[string | undefined, ActionResponse]> {
    if (!pools || !currentPool || !currentAccount || !fromAccount.walletTokens || !toAccount.walletTokens) {
      console.error('Accounts and/or pools not loaded');
      throw new Error();
    }

    const refreshInstructions: TransactionInstruction[] = [];
    const instructions: TransactionInstruction[] = [];
    const fromChange = tokenInputAmount.eq(fromAccount.poolPositions[currentPool.symbol].maxTradeAmounts.withdraw)
      ? PoolTokenChange.setTo(0)
      : PoolTokenChange.setTo(fromAccount.poolPositions[currentPool.symbol].depositBalance.sub(tokenInputAmount));
    const toChange = PoolTokenChange.setTo(
      toAccount.poolPositions[currentPool.symbol].depositBalance.add(tokenInputAmount)
    );
    try {
      // Refresh positions
      await currentPool.withMarginRefreshAllPositionPrices({
        instructions: refreshInstructions,
        pools: pools.tokenPools,
        marginAccount: fromAccount
      });
      await currentPool.withMarginRefreshAllPositionPrices({
        instructions: refreshInstructions,
        pools: pools.tokenPools,
        marginAccount: toAccount
      });

      // toAccount deposit position
      const toAccountDepositPosition = await currentPool.withGetOrRegisterDepositPosition({
        instructions,
        marginAccount: toAccount
      });
      await toAccount.withUpdatePositionBalance({ instructions, position: toAccountDepositPosition });

      // Withdraw and deposit ix
      await currentPool.withWithdraw({
        instructions,
        marginAccount: fromAccount,
        destination: fromAccount.walletTokens.map[currentPool.symbol].address,
        change: fromChange
      });
      await currentPool.withDeposit({
        instructions,
        marginAccount: toAccount,
        source: fromAccount.walletTokens.map[currentPool.symbol].address,
        change: toChange
      });
      const txId = await currentAccount.sendAll([chunks(11, refreshInstructions), instructions]);
      await actionRefresh();
      return [txId, ActionResponse.Success];
    } catch (err: any) {
      console.error(err);
      if (err.toString().includes('User rejected') || err.toString().includes('Failed to sign')) {
        return [undefined, ActionResponse.Cancelled];
      } else {
        return [undefined, ActionResponse.Failed];
      }
    }
  }

  return {
    airdrop,
    createAccount,
    deposit,
    withdraw,
    borrow,
    repay,
    splTokenSwap,
    transfer
  };
}
