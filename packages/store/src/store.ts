import BN from 'bn.js';
import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

import { APPLICATION_WS_EVENTS, JET_WS_EVENTS } from './events';
import { createMarketsSlice, MarketsSlice } from './slices/markets';
import { createPoolsSlice, PoolUpdate, type PoolsSlice } from './slices/pools';
import { createPricesSlice, PricesSlice } from './slices/prices';
import { createSettingsSlice, SettingsSlice } from './slices/settings';

export type JetStore = PoolsSlice & MarketsSlice & PricesSlice & SettingsSlice;

export const useJetStore = create<JetStore, [['zustand/devtools', never], ['zustand/persist', JetStore]]>(
  devtools(
    persist(
      (...a) => ({
        ...createPoolsSlice(...a),
        ...createMarketsSlice(...a),
        ...createPricesSlice(...a),
        ...createSettingsSlice(...a)
      }),
      {
        name: 'jet-state',
        onRehydrateStorage: () => console.log()
      }
    )
  )
);

const ws = new WebSocket(`${process.env.WS_API}`);

ws.onopen = () => {
  const subscriptionEvent: APPLICATION_WS_EVENTS = {
    type: 'SUBSCRIBE',
    payload: {
      wallet: 'APhQTneeYjR8A5E3BuJBZFjHKpWdxHhTdiE1nuzoT553',
      margin_accounts: ['B8Tifsx1p22hto44FBo3sEt5nJmHtzTUNbM4f9UP42GV', 'GT7eBGzue4e1Bq7N3Qox518nsCfyzEkEZeKwpD2vQMVM']
    }
  };
  ws.send(JSON.stringify(subscriptionEvent));
};

ws.onmessage = (msg: MessageEvent<string>) => {
  const data: JET_WS_EVENTS = JSON.parse(msg.data);

  if (data.type === 'MARGIN-POOL-UPDATE') {
    const update: PoolUpdate = {
      address: data.payload.address,
      borrowed_tokens: new BN(data.payload.borrowed_tokens).toNumber(),
      deposit_tokens: new BN(data.payload.deposit_tokens).toNumber(),
      deposit_notes: new BN(data.payload.deposit_notes).toNumber(),
      accrued_until: new Date(data.payload.accrued_until * 1000)
    };
    useJetStore.getState().updatePool(update);
  } else if (data.type === 'PRICE-UPDATE') {
    useJetStore.getState().updatePrices(data);
  }
};
