import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';

export const createAccountsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], AccountsSlice> = (
  set,
  get
) => ({
  wallets: {},
  selectedWallet: null,
  airspaceLookupTables: [],
  connectWallet: async wallet => {
    set(
      () => {
        const cluster = get().settings.cluster;
        initWebsocket(cluster, wallet);
        return { selectedWallet: wallet };
      },
      false,
      'CONNECT_WALLET'
    );
  },
  disconnectWallet: () =>
    set(
      () => ({
        selectedWallet: null
      }),
      false,
      'DISCONNECT_WALLET'
    ),
  updateLookupTables: tables => set(() => ({ airspaceLookupTables: tables }), false, 'UPDATE_LOOKUP_TABLE_ADDRESSES'),
  updateMarginAccount: (update: MarginAccountData) => {
    return set(
      state => {
        if (!state.selectedWallet) {
          return state;
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
                  ...account
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
      state => {
        if (!state.selectedWallet) {
          return state;
        }
        const wallet = state.wallets[state.selectedWallet];
        return {
          ...state,
          wallets: {
            ...state.wallets,
            [state.selectedWallet]: {
              ...wallet,
              accounts: update,
              selectedMarginAccount: keys.includes(String(wallet.selectedMarginAccount))
                ? wallet.selectedMarginAccount
                : keys[0]
            }
          }
        };
      },
      true,
      'INIT_MARGIN_ACCOUNTS'
    );
  },
  selectMarginAccount: (address: string) => {
    return set(
      state => {
        if (!state.selectedWallet) {
          return state;
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
        };
      },
      false,
      'SELECT_MARGIN_ACCOUNT'
    );
  }
});
