import { useEffect, useMemo } from 'react';
import { atom, useRecoilState, useRecoilValue, useResetRecoilState, useSetRecoilState } from 'recoil';
import axios from 'axios';
import { LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { Order } from '@project-serum/serum/lib/market';
import { useWallet } from '@solana/wallet-adapter-react';
import {
  MarginAccount,
  MarginClient,
  AccountTransaction,
  OrderSide,
  OrderType,
  OrderStatus,
  TokenAmount
} from '@jet-lab/margin';
import { localStorageEffect } from '../effects/localStorageEffect';
import { Cluster } from '../settings/settings';
import { Dictionary } from '../settings/localization/localization';
import { ActionRefresh, ACTION_REFRESH_INTERVAL } from '../actions/actions';
import { WalletInit, WalletTokens } from './walletTokens';
import { Pools } from '../borrow/pools';
import { useProvider } from '../../utils/jet/provider';

export interface AccountOrder {
  signature: string;
  timestamp: number;
  dateString?: string;
  pair: string;
  side: OrderSide;
  size: number;
  filledSize: number;
  price: number;
  aveFillPrice: number;
  type: OrderType;
  status: OrderStatus;
  totalFees: number;
  serumFeeTier: number;
  serumFees: number;
  solanaFees: number;
  serumOrder: Order;
}
export interface AccountHistory {
  orders: AccountOrder[];
  transactions: AccountTransaction[];
}
export interface AccountBalance {
  tokenName: string;
  tokenSymbol: string;
  depositBalance: TokenAmount;
  loanBalance: TokenAmount;
  netBalance: TokenAmount;
  inOrders: Record<string, number>;
  fiatValue: string;
  percentageOfPortfolio: number;
  depositRate: number;
  borrowRate: number;
}
export const Accounts = atom({
  key: 'accounts',
  default: [] as MarginAccount[],
  dangerouslyAllowMutability: true
});
export const CurrentAccount = atom({
  key: 'currentAccount',
  default: undefined as MarginAccount | undefined,
  dangerouslyAllowMutability: true
});
export const AccountNames = atom({
  key: 'accountNames',
  default: {} as Record<string, string>
});
export const CurrentAccountName = atom({
  key: 'currentAccountName',
  default: undefined as string | undefined,
  effects: [localStorageEffect('jetAppCurrentAccountName')]
});
export const FavoriteAccounts = atom({
  key: 'favoriteAccounts',
  default: {} as Record<string, string[]>,
  effects: [localStorageEffect('jetAppFavoritedAccounts')],
  dangerouslyAllowMutability: true
});
export const AccountHistories = atom({
  key: 'AccountHistories',
  default: {} as Record<string, AccountHistory | undefined>
});
export const NewAccountRentFee = atom({
  key: 'newAccountRentFee',
  default: 0 as number
});
export const AccountsLoading = atom({
  key: 'accountsLoading',
  default: false as boolean
});
export const AccountsInit = atom({
  key: 'accountsInit',
  default: false as boolean
});

// Wrapper to provide contextual updates to Accounts
export function AccountsWrapper(props: { children: JSX.Element }) {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const { programs, provider } = useProvider();
  const { publicKey } = useWallet();
  const walletParam = new URLSearchParams(document.location.search).get('wallet');
  const owner = useMemo(() => publicKey ?? (walletParam ? new PublicKey(walletParam) : null), [publicKey, walletParam]);
  const pools = useRecoilValue(Pools);
  const walletTokens = useRecoilValue(WalletTokens);
  const walletInit = useRecoilValue(WalletInit);
  const [accountNames, setAccountNames] = useRecoilState(AccountNames);
  const [currentAccountName, setCurrentAccountName] = useRecoilState(CurrentAccountName);
  const setAccountHistories = useSetRecoilState(AccountHistories);
  const setNewAccountRentFee = useSetRecoilState(NewAccountRentFee);
  const setAccountsLoading = useSetRecoilState(AccountsLoading);
  const setAccountsInit = useSetRecoilState(AccountsInit);
  const [accounts, setAccounts] = useRecoilState(Accounts);
  // Reset the state of margin accounts
  const resetAccounts = useResetRecoilState(Accounts);
  const [currentAccount, setCurrentAccount] = useRecoilState(CurrentAccount);
  const resetCurrentAccount = useResetRecoilState(CurrentAccount);
  const actionRefresh = useRecoilValue(ActionRefresh);

  // Update currentAccount when currentAccountName changes
  useEffect(() => {
    const accountMatch = Object.keys(accountNames).filter(key => accountNames[key] === currentAccountName)[0];
    if (accountMatch) {
      setCurrentAccount(accounts.filter(account => account.address.toString() === accountMatch)[0]);
    }
  }, [accountNames, accounts, currentAccountName, setCurrentAccount]);

  // Set rent fee for creating a new account
  useEffect(() => {
    async function getNewAccountRentFee() {
      if (!programs) {
        return;
      }

      const rentFeeLamports = await programs.connection.getMinimumBalanceForRentExemption(
        programs.margin.account.marginAccount.size
      );
      const rentFee = rentFeeLamports / LAMPORTS_PER_SOL;
      setNewAccountRentFee(rentFee);
    }
    getNewAccountRentFee();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [programs]);

  // Fetch all margin accounts on wallet init
  // Re-fetch upon an actionRefresh
  useEffect(() => {
    // Get all margin accounts
    async function getAccounts() {
      setAccountsLoading(true);
      if (!pools || !owner || !programs) {
        resetAccounts();
        resetCurrentAccount();
        setAccountsInit(false);
        return;
      }

      // Load accounts, only use ones that exist
      const accounts: MarginAccount[] = [];
      try {
        const allAccounts = await MarginAccount.loadAllByOwner({
          programs,
          provider,
          pools: pools.tokenPools,
          walletTokens,
          owner
        });
        for (const account of allAccounts) {
          const exists = await account.exists();
          if (exists) {
            accounts.push(account);
          }
        }
      } catch (err) {
        console.error(err);
      }

      // Set up accountNames and set up histories
      accounts.sort((a, b) => (a.seed > b.seed ? 1 : -1));
      const accountNames = await getAccountNames(owner);
      const sortedAccountNames: Record<string, string> = {};
      const accountHistories: Record<string, AccountHistory | undefined> = {};
      for (const account of accounts) {
        const accountKey = account.address.toString();
        sortedAccountNames[accountKey] = accountNames[accountKey] ?? `${dictionary.common.account} ${account.seed + 1}`;

        const accountHistory = await getAccountHistory(account.address);
        accountHistories[account.address.toString()] = accountHistory;

        // If account is currently being liquidated, switch to that account
        if (account.isBeingLiquidated) {
          setCurrentAccountName(sortedAccountNames[accountKey]);
        }
      }
      // If no currentAccount select first
      if (!currentAccount && accounts.length) {
        setCurrentAccountName(sortedAccountNames[accounts[0].address.toString()]);
      }

      setAccounts(accounts);
      setAccountNames(sortedAccountNames);
      setAccountHistories(accountHistories);
      setAccountsInit(true);
      setAccountsLoading(false);
    }

    // Fetch account names for owner address
    async function getAccountNames(owner: PublicKey): Promise<Record<string, string>> {
      let accountNames = {};
      try {
        const resp = await axios.get(
          `https://api.jetprotocol.io/v1/margin/${owner.toString()}/accounts${
            cluster === 'devnet' ? '?network=devnet' : ''
          }`
        );
        if (resp.status !== 404) {
          accountNames = resp.data;
        }
      } catch (err) {
        // Ignore this error
      }

      return accountNames;
    }

    // Fetch account history for an account publicKey
    async function getAccountHistory(accountKey: PublicKey): Promise<AccountHistory | undefined> {
      if (!pools || !owner) {
        return;
      }

      // TODO: Fetch account orders
      const orders: AccountOrder[] = [];
      const mints: any = {};
      for (const pool of Object.values(pools.tokenPools)) {
        mints[pool.symbol] = {
          tokenMint: pool.addresses.tokenMint,
          depositNoteMint: pool.addresses.depositNoteMint,
          loanNoteMint: pool.addresses.loanNoteMint
        };
      }
      const transactions = await MarginClient.getTransactionHistory(provider, accountKey, mints, cluster);
      return {
        orders,
        transactions
      };
    }

    if (pools && owner) {
      getAccounts();
      const accountsInterval = setInterval(getAccounts, ACTION_REFRESH_INTERVAL);
      return () => clearInterval(accountsInterval);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [provider.connection, pools, walletInit, publicKey, actionRefresh]);

  return <>{props.children}</>;
}

// Get a margin account from a given account name
export function useAccountFromName(accountName: string | undefined): MarginAccount | undefined {
  const accountNames = useRecoilValue(AccountNames);
  const accounts = useRecoilValue(Accounts);

  return useMemo(() => {
    let marginAccount: MarginAccount | undefined = undefined;
    const accountMatch = Object.keys(accountNames).filter(key => accountNames[key] === accountName)[0];
    if (accountMatch) {
      marginAccount = accounts.filter(account => account.address.toString() === accountMatch)[0];
    }

    return marginAccount;
  }, [accounts, accountNames, accountName]);
}

// Get a list of account names to select from
export function useAccountNames(): string[] {
  const accountNames = useRecoilValue(AccountNames);

  const names: string[] = [];
  for (const name of Object.values(accountNames)) {
    names.push(name);
  }
  return names;
}

// Use the current margin account's history
export function CurrentAccountHistory(): AccountHistory | undefined {
  const currentAccountName = useRecoilValue(CurrentAccountName);
  const accountNames = useRecoilValue(AccountNames);
  const accountHistories = useRecoilValue(AccountHistories);

  return useMemo(() => {
    let currentAccountHistory: AccountHistory | undefined = accountHistories[Object.keys(accountHistories)[0]];
    for (const accountKey of Object.keys(accountNames)) {
      if (accountNames[accountKey] === currentAccountName) {
        currentAccountHistory = accountHistories[accountKey];
      }
    }
    return currentAccountHistory;
  }, [accountHistories, accountNames, currentAccountName]);
}
