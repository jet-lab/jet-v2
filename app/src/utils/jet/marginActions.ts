import { useRecoilValue, useSetRecoilState } from 'recoil';
import axios from 'axios';
import { TransactionInstruction } from '@solana/web3.js';
import { Order } from '@project-serum/serum/lib/market';
import { NATIVE_MINT } from '@solana/spl-token';
import { useWallet } from '@solana/wallet-adapter-react';
import { MarginAccount, Pool, PoolTokenChange, sleep, TokenAmount, TokenFaucet, TokenFormat } from '@jet-lab/margin';
import { Cluster } from '../../state/settings/settings';
import { Pools, CurrentPool } from '../../state/borrow/pools';
import { CurrentMarket } from '../../state/trade/market';
import { WalletTokens } from '../../state/user/walletTokens';
import { CurrentAccount } from '../../state/user/accounts';
import { TokenInputAmount, ActionRefresh } from '../../state/actions/actions';
import { OrderSide, OrderType, OrderPrice, OrderSize } from '../../state/trade/order';
import { useProvider } from './provider';
import { useMarginConfig } from './marginConfig';

export enum ActionResponse {
  Success = 'SUCCESS',
  Failed = 'FAILED',
  Cancelled = 'CANCELLED'
}
export function useMarginActions() {
  const cluster = useRecoilValue(Cluster);
  const config = useMarginConfig();
  const { programs, provider } = useProvider();
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const currentMarket = useRecoilValue(CurrentMarket);
  const wallet = useWallet();
  const walletTokens = useRecoilValue(WalletTokens);
  const currentAccount = useRecoilValue(CurrentAccount);
  const accountPoolPosition = currentPool?.symbol && currentAccount?.poolPositions[currentPool.symbol];
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const orderSide = useRecoilValue(OrderSide);
  const orderType = useRecoilValue(OrderType);
  const orderPrice = useRecoilValue(OrderPrice);
  const orderSize = useRecoilValue(OrderSize);
  const setActionRefresh = useSetRecoilState(ActionRefresh);

  // Refresh to trigger new data fetching after 2 seconds
  async function actionRefresh() {
    await sleep(3000);
    setActionRefresh(true);
    setActionRefresh(false);
  }

  // If on devnet, user can airdrop themself tokens
  async function airdrop(pool: Pool): Promise<[string | undefined, ActionResponse]> {
    if (!config) {
      throw new Error('No Config');
    }
    if (!programs) {
      throw new Error('No Programs');
    }
    if (!wallet.publicKey) {
      throw new Error('No Public Key');
    }

    // If SOL, only airdrop 1 token
    let amount = TokenAmount.tokens('100', pool.decimals);
    if (pool.addresses.tokenMint.equals(NATIVE_MINT)) {
      amount = TokenAmount.tokens('1', pool.decimals);
    }

    const token = config.tokens[pool.symbol];
    try {
      const txId = await TokenFaucet.airdrop(programs, provider, amount.lamports, token, wallet.publicKey);
      await actionRefresh();
      return [txId, ActionResponse.Success];
    } catch (err) {
      console.error(err);
      return [undefined, ActionResponse.Failed];
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
      if (accountName) {
        axios
          .put(`https://api.jetprotocol.io/v1/margin/${wallet.publicKey?.toString()}/accounts`, {
            alias: accountName,
            network: cluster,
            publicKey: newMarginAccount.address.toString()
          })
          .catch(err => err);
      }

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
        change: PoolTokenChange.shiftBy(accountPoolPosition.loanBalance.add(tokenInputAmount)),
        destination: TokenFormat.unwrappedSol
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
  async function repay(sourceAccount?: MarginAccount): Promise<[string | undefined, ActionResponse]> {
    if (!pools || !currentPool || !walletTokens || !currentAccount || !accountPoolPosition) {
      console.error('Accounts and/or pools not loaded');
      throw new Error();
    }

    const closeLoan = tokenInputAmount.eq(accountPoolPosition.maxTradeAmounts.repay);
    const change = closeLoan
      ? PoolTokenChange.setTo(0)
      : PoolTokenChange.setTo(accountPoolPosition.loanBalance.sub(tokenInputAmount));
    try {
      const txId = await currentPool.marginRepay({
        marginAccount: currentAccount,
        source: sourceAccount
          ? sourceAccount.poolPositions[currentPool.symbol].depositPosition?.address
          : walletTokens.map[currentPool.symbol].address,
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
  async function swap(
    inputToken: Pool,
    outputToken: Pool,
    swapAmount: TokenAmount,
    minAmountOut: TokenAmount
  ): Promise<[string | undefined, ActionResponse]> {
    if (!pools || !inputToken || !outputToken || !currentAccount) {
      console.error('Input/output tokens or current account undefined');
      throw new Error();
    }

    try {
      const txId = await inputToken.swap({
        marginAccount: currentAccount,
        pools: Object.values(pools.tokenPools),
        outputToken,
        swapAmount,
        minAmountOut
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

  // Transfer
  async function transfer(
    fromAccount: MarginAccount,
    toAccount: MarginAccount
  ): Promise<[string | undefined, ActionResponse]> {
    if (!pools || !currentPool || !currentAccount || !fromAccount.walletTokens || !toAccount.walletTokens) {
      console.error('Accounts and/or pools not loaded');
      throw new Error();
    }

    const instructions: TransactionInstruction[] = [];
    const fromChange = tokenInputAmount.eq(fromAccount.poolPositions[currentPool.symbol].maxTradeAmounts.withdraw)
      ? PoolTokenChange.setTo(0)
      : PoolTokenChange.setTo(fromAccount.poolPositions[currentPool.symbol].depositBalance.sub(tokenInputAmount));
    const toChange = PoolTokenChange.setTo(
      toAccount.poolPositions[currentPool.symbol].depositBalance.add(tokenInputAmount)
    );
    try {
      await currentPool.withWithdraw({
        instructions,
        marginAccount: fromAccount,
        destination: fromAccount.walletTokens.map[currentPool.symbol].address,
        change: fromChange
      });
      await currentPool.withDeposit({
        instructions,
        marginAccount: fromAccount,
        source: fromAccount.walletTokens.map[currentPool.symbol].address,
        change: toChange
      });
      const txId = await currentAccount.sendAndConfirm(instructions);
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

  // Place order
  async function placeOrder(): Promise<[string | undefined, ActionResponse]> {
    if (!currentMarket || !currentAccount) {
      console.error('Accounts and/or market not loaded');
      throw new Error();
    }

    try {
      const txId = await currentMarket.placeOrder({
        marginAccount: currentAccount,
        orderSide,
        orderType,
        orderPrice,
        orderSize
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

  // Cancel order
  async function cancelOrder(order: Order): Promise<[string | undefined, ActionResponse]> {
    if (!currentMarket || !currentAccount) {
      console.error('Accounts and/or market not loaded');
      throw new Error();
    }

    try {
      const txId = await currentMarket.cancelOrder({ marginAccount: currentAccount, order });
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
    swap,
    transfer,
    placeOrder,
    cancelOrder
  };
}
