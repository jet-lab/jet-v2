import { MarginAccount } from '@jet-lab/margin';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';

interface WalletToken {
  address: string;
  symbol: string;
  amount: number;
}

interface Wallet {
  pubkey: string;
  accounts: Record<string, MarginAccount>;
  tokens: Record<string, WalletToken>;
}

export interface AccountsSlice {
  accounts: Record<string, Wallet>;
  updateAccounts: (update) => void;
}

export const createAccountsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], AccountsSlice> = set => ({
  accounts: {},
  updateAccounts: update => set(state => state, false, 'UPDATE_ACCOUNTS')
});
