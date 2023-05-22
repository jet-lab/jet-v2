import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

import { createPoolsSlice } from './slices/pools';
import { createPricesSlice } from './slices/prices';
import { createSettingsSlice } from './slices/settings';
import { createAccountsSlice } from './slices/accounts';
import { initWebsocket } from './websocket';

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
