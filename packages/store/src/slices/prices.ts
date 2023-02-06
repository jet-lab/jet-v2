import { StateCreator } from 'zustand';
import { PRICE_UPDATE, TOKEN_PRICE_UPDATE } from '../events';
import { JetStore } from '../store';

export interface PricesSlice {
  prices: Record<string, TOKEN_PRICE_UPDATE>;
  updatePrices: (update: PRICE_UPDATE) => void;
}

export interface PoolUpdate {
  address: string;
  borrowed_tokens: number;
  deposit_tokens: number;
  deposit_notes: number;
  accrued_until: Date;
}

export const createPricesSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], PricesSlice> = set => ({
  prices: {},
  updatePrices: (update: PRICE_UPDATE) => set(() => ({ prices: update.payload }), false, 'UPDATE_PRICE')
});
