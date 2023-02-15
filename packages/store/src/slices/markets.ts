import { FixedTermMarket } from '@jet-lab/margin';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';

export interface MarketsSlice {
  markets: Record<string, FixedTermMarket>;
}

interface MarketUpdate {
  address: string;
}

export const createMarketsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], MarketsSlice> = set => ({
  markets: {},
  updateMarket: (update: MarketUpdate) =>
    set(
      state => {
        console.log('MARKET UPDATE: ', update);
        return { ...state, markets: { ...state.markets } };
      },
      false,
      'UPDATE_MARKET'
    )
});
