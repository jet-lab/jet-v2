import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

import { createPoolsSlice, type PoolsSlice } from './slices/pools';
import { createPricesSlice, PricesSlice } from './slices/prices';
import { createSettingsSlice, SettingsSlice } from './slices/settings';
import { AccountsSlice, createAccountsSlice } from './slices/accounts';
import { initWebsocket } from './websocket';

export type JetStore = PoolsSlice & PricesSlice & SettingsSlice & AccountsSlice;

export const useJetStore = create<JetStore, [['zustand/devtools', never], ['zustand/persist', JetStore]]>(
  devtools(
    persist(
      (...a) => ({
        ...createPoolsSlice(...a),
        ...createPricesSlice(...a),
        ...createSettingsSlice(...a),
        ...createAccountsSlice(...a)
      }),
      {
        name: 'jet-state',
        onRehydrateStorage: () => {
          return state => state && initWebsocket(state.settings.cluster, state.selectedWallet);
        }
      }
    )
  )
);
