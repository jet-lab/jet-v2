import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';

export const createAccountsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], AccountsSlice> = (
  set,
  get
) => ({
  wallets: {},
  selectedWallet: null,
  marginAccounts: {},
  selectedMarginAccount: null,
  marginAccountLookupTables: {},
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
  updateAirspaceLookupTables: tables =>
    set(() => ({ airspaceLookupTables: tables }), false, 'UPDATE_LOOKUP_TABLE_ADDRESSES'),
  updateMarginAccount: (update: MarginAccountData) => {
    return set(
      state => {
        return {
          ...state,
          marginAccounts: {
            ...state.marginAccounts,
            [update.address]: {
              ...update
            }
          }
        };
      },
      true,
      'UPDATE_MARGIN_ACCOUNT'
    );
  },
  initAllMarginAccounts: (update: Record<string, MarginAccountData>) => {
    // on init select first margin account if no other margin account is selected
    const keys = Object.keys(update);
    return set(
      state => {
        return {
          ...state,
          marginAccounts: update,
          selectedMarginAccount: keys.includes(String(state.selectedMarginAccount))
            ? state.selectedMarginAccount
            : keys[0]
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
        const keys = Object.keys(state.marginAccounts);
        return {
          ...state,
          selectedMarginAccount: keys.includes(String(address)) ? address : keys[0]
        };
      },
      false,
      'SELECT_MARGIN_ACCOUNT'
    );
  },
  updateMarginAccountLookupTables: (address: string, tables: LookupTable[]) => {
    return set(
      state => {
        return {
          ...state,
          marginAccountLookupTables: {
            ...state.marginAccountLookupTables,
            [address]: tables
          }
        };
      },
      false,
      'UPDATE_MARGIN_ACCOUNT_LOOKUP_TABLES'
    );
  }
});
