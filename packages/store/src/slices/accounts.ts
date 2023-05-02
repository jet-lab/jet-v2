import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';

// interface WalletToken {
//   address: string;
//   symbol: string;
//   amount: number;
// }

interface MarginAccountData {
  address: string;
  owner: string;
  liquidator: string;
  positions: MarginPosition[];
  // Seems reasonable to store this here as it's tied to the margin account
  lookupTables: string[];
}

interface MarginPosition {
  adapter: string;
  address: string;
  balance: number;
  balanceTimestamp: number;
  exponent: number;
  kind: 'Collateral' | 'AdapterCollateral' | 'Claim';
  maxStaleness: number;
  price: {
    exponent: number;
    isValid: number;
    timestamp: number;
    value: number;
  }
  token: string;
  value: string; // Number192 formatted as decimal string
  valueModifier: number;
}

interface Wallet {
  pubkey: string;
  accounts: Record<string, MarginAccountData>;
  selectedMarginAccount: string | null;
  // tokens: Record<string, WalletToken>;
}

export interface AccountsSlice {
  wallets: Record<string, Wallet>;
  selectedWallet: string | null;
  connectWallet: (wallet: string) => void;
  disconnectWallet: () => void;
  // The lookup addresses of the airspace. Only storing here as we don't yet have an airspace slice
  airspaceLookupTableAddresses: string[];
  updateLookupTableAddresses: (addresses: string[]) => void;
  updateMarginAccount: (update: MarginAccountData) => void;
  initAllMarginAccounts: (update: Record<string, MarginAccountData>) => void;
  selectMarginAccount: (address: string) => void;
}

export const createAccountsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], AccountsSlice> = (set, get) => ({
  wallets: {},
  selectedWallet: null,
  airspaceLookupTableAddresses: [],
  connectWallet: async wallet => {
    set(() => {
      const cluster = get().settings.cluster
      initWebsocket(cluster, wallet)
      return ({ selectedWallet: wallet })
    }, false, 'CONNECT_WALLET');
  },
  disconnectWallet: () => set(() => ({
    selectedWallet: null,
  }), false, 'DISCONNECT_WALLET'),
  updateLookupTableAddresses: addresses => set(() => ({ airspaceLookupTableAddresses: addresses }), false, 'UPDATE_LOOKUP_TABLE_ADDRESSES'),
  updateMarginAccount: (update: MarginAccountData) => {
    return set(
      state => {
        if (!state.selectedWallet) {
          return state
        }
        const wallet = state.wallets[state.selectedWallet];
        const account = wallet.accounts[update.address];
        return {
          ...state,
          wallets: {
            ...state.wallets,
            [state.selectedWallet]: {
              ...wallet,
              accounts: {
                ...wallet.accounts,
                [update.address]: {
                  ...account,
                }
              }
            }
          },
        };
      },
      false, 'UPDATE_MARGIN_ACCOUNT'
    );
  },
  initAllMarginAccounts: (update: Record<string, MarginAccountData>) => {
    // on init select first margin account if no other margin account is selected
    const keys = Object.keys(update);
    return set(state => {
      if (!state.selectedWallet) {
        return state
      }
      const wallet = state.wallets[state.selectedWallet];
      return {
        ...state,
        wallets: {
          ...state.wallets,
          [state.selectedWallet]: {
            ...wallet,
            accounts: update,
            selectedMarginAccount: keys.includes(String(wallet.selectedMarginAccount)) ? wallet.selectedMarginAccount : keys[0]
          }
        }
      }
    }, true, 'INIT_MARGIN_ACCOUNTS')
  },
  selectMarginAccount: (address: string) => {
    return set(state => {
      if (!state.selectedWallet) {
        return state
      }
      const wallet = state.wallets[state.selectedWallet];
      const keys = Object.keys(wallet.accounts);
      return {
        ...state,
        wallets: {
          ...state.wallets,
          [state.selectedWallet]: {
            ...wallet,
            selectedMarginAccount: keys.includes(String(address)) ? address : keys[0]
          }
        }
      }
    }, false, 'SELECT_MARGIN_ACCOUNT')
  }
});
