import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';


// interface WalletToken {
//   address: string;
//   symbol: string;
//   amount: number;
// }

interface Wallet {
  pubkey: string;
  accounts: Record<string, MarginAccountData>;
  selectedMarginAccount: string;
  // tokens: Record<string, WalletToken>;
}

export interface AccountsSlice {
  accounts: Record<string, Wallet>;
  selectedWallet: string | null;
  connectWallet: (wallet: string) => void;
  disconnectWallet: () => void;
  updateMarginAccount: (update: MarginAccountUpdate) => void;
  initAllMarginAccounts: (update: Record<string, MarginAccountData>) => void;
  selectMarginAccount: (address: string) => void;
}

export const createAccountsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], AccountsSlice> = (set, get) => ({
  accounts: {},
  selectedWallet: null, // TODO: maybe we can mak this non-nullable?
  connectWallet: async wallet => {
    set(() => {
      const cluster = get().settings.cluster
      initWebsocket(cluster, wallet)
      return ({ selectedWallet: wallet })
    }, false, 'CONNECT_WALLET');
  },
  disconnectWallet: () => set(() => ({ selectedWallet: null }), false, 'DISCONNECT_WALLET'),
  updateMarginAccount: (update: MarginAccountUpdate) => {
    return set(
      state => {
        if (!state.selectedWallet) {
          return state
        }
        const wallet = state.accounts[state.selectedWallet]
        const account = wallet.accounts[update.address];
        return {
          accounts: {
            ...state.accounts,
            [state.selectedWallet]: {
              ...wallet,
              accounts: {
                ...wallet.accounts,
                [update.address]: {
                  ...account,
                }
              }
            }
          }
        };
      },
      false,
      'UPDATE_MARGIN_ACCOUNT'
    );
  },
  initAllMarginAccounts: (update: Record<string, MarginAccountData>) => {
    // on init select first margin account if no other margin account is selected
    const keys = Object.keys(update);
    return set(
      state => ({
        ...state,
        marginAccounts: update,
        selectedAccountKey: keys.includes(String(state.selectMarginAccount)) ? state.selectMarginAccount : keys[0]
      }),
      true,
      'INIT_MARGIN_ACCOUNTS'
    );
  },
  selectMarginAccount: (address: string) => set(() => ({ selectedPoolKey: address }), false, 'SELECT_MARGIN_ACCOUNT')
});


export interface MarginAccountData {
  address: string;
  owner: string;
  liquidator: string;
  positions: MarginPosition[],
}

export interface MarginAccountUpdate {
  address: string;
  liquidator: string;
  positions: MarginPosition[];
}

export interface MarginPosition {
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
