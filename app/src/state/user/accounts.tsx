import { useEffect, useMemo } from 'react';
import { atom, selector, useRecoilState, useRecoilValue, useResetRecoilState, useSetRecoilState } from 'recoil';
// import axios from 'axios';
import { PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { MarginAccount, MarginClient, AccountTransaction, TokenAmount } from '@jet-lab/margin';
import { localStorageEffect } from '../effects/localStorageEffect';
import { Cluster } from '../settings/settings';
import { Dictionary } from '../settings/localization/localization';
import { ActionRefresh, ACTION_REFRESH_INTERVAL } from '../actions/actions';
import { walletParam, WalletTokens } from './walletTokens';
import { Pools } from '../pools/pools';
import { useProvider } from '@utils/jet/provider';
import { MarginConfig } from '../config/marginConfig';

// Interfaces for account order and tx history
export interface AccountHistory {
  transactions: AccountTransaction[];
}
// Interface for a particular token's balances associated with an account
export interface AccountBalance {
  tokenName: string;
  tokenSymbol: string;
  depositBalance: TokenAmount;
  loanBalance: TokenAmount;
  netBalance: TokenAmount;
  fiatValue: string;
  percentageOfPortfolio: number;
  depositRate: number;
  borrowRate: number;
}

// State of all margin accounts associated with the connected wallet
export const Accounts = atom({
  key: 'accounts',
  default: [] as MarginAccount[],
  dangerouslyAllowMutability: true
});
// To know when we are loading in account data
export const AccountsLoading = atom({
  key: 'accountsLoading',
  default: false as boolean
});
// Track the current account by its name, so it's lightweight
// and we can reference this value to select the entire state
export const CurrentAccountAddress = atom({
  key: 'currentAccountAddress',
  default: '' as string,
  effects: [localStorageEffect('jetAppCurrentAccountAddress')]
});
// User's starred accounts for quick selection
export const FavoriteAccounts = atom({
  key: 'favoriteAccounts',
  default: {} as Record<string, string[]>,
  effects: [localStorageEffect('jetAppFavoritedAccounts')],
  dangerouslyAllowMutability: true
});
// Record of account pubkeys to their name for selection
export const AccountNames = atom({
  key: 'accountNames',
  default: {} as Record<string, string>
});

// Select the current account's state
export const CurrentAccount = selector<MarginAccount | undefined>({
  key: 'currentAccount',
  get: ({ get }) => {
    const accounts = get(Accounts);
    const currentAddress = get(CurrentAccountAddress);

    const currentAccount = Object.values(accounts).filter(account => account.address.toString() === currentAddress)[0];
    return currentAccount;
  },
  dangerouslyAllowMutability: true
});
// Fetch / select the current account's order and tx history
export const CurrentAccountHistory = atom({
  key: 'currentAccountHistory',
  default: undefined as AccountHistory | undefined,
  dangerouslyAllowMutability: true
});
// Loading account history
export const AccountHistoryLoaded = atom({
  key: 'accountHistoryLoaded',
  default: false as boolean
});

// A syncer to be called so that we can have dependent atom state
export function useAccountsSyncer() {
  const cluster = useRecoilValue(Cluster);
  const marginConfig = useRecoilValue(MainConfig);
  const dictionary = useRecoilValue(Dictionary);
  const { programs, provider } = useProvider();
  const { publicKey } = useWallet();
  const owner = publicKey ?? (walletParam ? new PublicKey(walletParam) : null);
  const pools = useRecoilValue(Pools);
  const walletTokens = useRecoilValue(WalletTokens);
  const setAccountNames = useSetRecoilState(AccountNames);
  const [currentAccountAddress, setCurrentAccountAddress] = useRecoilState(CurrentAccountAddress);
  const setAccounts = useSetRecoilState(Accounts);
  const resetAccounts = useResetRecoilState(Accounts);
  const currentAccount = useRecoilValue(CurrentAccount);
  const setCurrentAccountHistory = useSetRecoilState(CurrentAccountHistory);
  const setAccountsLoading = useSetRecoilState(AccountsLoading);
  const setAccountHistoryLoaded = useSetRecoilState(AccountHistoryLoaded);
  const actionRefresh = useRecoilValue(ActionRefresh);

  // When we change address
  useEffect(() => setAccountHistoryLoaded(false), [currentAccountAddress, setAccountHistoryLoaded]);

  // Fetch all margin accounts on wallet init
  useEffect(() => {
    // Get all margin accounts
    async function getAccounts() {
      if (!pools || !owner || !programs) {
        resetAccounts();
        return;
      }

      // Load accounts, only use ones that exist
      setAccountsLoading(true);
      const accounts: MarginAccount[] = [];
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

      // Set up accountNames and set up histories
      accounts.sort((a, b) => (a.seed > b.seed ? 1 : -1));
      //const accountNames = await getAccountNames(owner);
      const sortedAccountNames: Record<string, string> = {};
      for (const account of accounts) {
        const accountKey = account.address.toString();
        sortedAccountNames[accountKey] = /* accountNames[accountKey] ?? */ `${dictionary.common.account} ${
          account.seed + 1
        }`;

        // If account is currently being liquidated, switch to that account
        if (account.isBeingLiquidated) {
          setCurrentAccountAddress(accountKey);
        }
      }
      // If no currentAccount select first
      if (!currentAccount && accounts.length) {
        setCurrentAccountAddress(accounts[0].address.toString());
      }

      setAccounts(accounts);
      setAccountNames(sortedAccountNames);
      setAccountsLoading(false);
    }

    // TODO: add account names back in
    // Fetch account names for owner address
    // async function getAccountNames(owner: PublicKey): Promise<Record<string, string>> {
    //   let accountNames = {};
    //   try {
    //     const resp = await axios.get(
    //       ``
    //     );
    //     if (resp.status !== 404) {
    //       accountNames = resp.data;
    //     }
    //   } catch (err) {
    //     // Ignore this error
    //   }

    //   return accountNames;
    // }

    // Fetch accounts on an interval
    getAccounts();
    const accountsInterval = setInterval(getAccounts, ACTION_REFRESH_INTERVAL);
    return () => clearInterval(accountsInterval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pools, owner, provider.connection, actionRefresh]);

  // Update current account history
  useEffect(() => {
    async function getAccountHistory() {
      if (!pools || !currentAccount) {
        return;
      }

      const mints: any = {};
      for (const pool of Object.values(pools.tokenPools)) {
        mints[pool.symbol] = {
          tokenMint: pool.addresses.tokenMint,
          depositNoteMint: pool.addresses.depositNoteMint,
          loanNoteMint: pool.addresses.loanNoteMint
        };
      }

      // Account trasactions
      const transactions =
        cluster !== 'mainnet-beta'
          ? await MarginClient.getTransactionHistory(
              currentAccount.provider,
              currentAccount.address,
              mints,
              marginConfig
            )
          : await MarginClient.getBlackBoxHistory(currentAccount.address, cluster);

      setAccountHistoryLoaded(true);
      return {
        transactions
      };
    }

    getAccountHistory().then(history => {
      const accountHistory = history;
      if (accountHistory) {
        accountHistory.transactions = accountHistory.transactions.sort((a, b) => (a.timestamp < b.timestamp ? 1 : -1));
        setCurrentAccountHistory(accountHistory);
      }
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentAccount, provider.connection, actionRefresh]);

  return <></>;
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
