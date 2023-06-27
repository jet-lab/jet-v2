import { StateCreator } from 'zustand';

export const createFixedTermSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], FixedTermSlice> = set => ({
  markets: {},
  selectedMarketKey: '',
  marketsLastUpdated: 0,
  openOrders: {},
  openPositions: {},
  updateMarket: (_update: {}) => {
    return set(
      state => state,
      false,
      'UPDATE_MARKET'
    );
  },
  initAllMarkets: (update: Record<string, {}>) => {
    // on init select first market if no other market is selected
    // const keys = Object.keys(update);
    return set(
      state => ({
        ...state,
        markets: update,
        // selectedMarketKey: keys.includes(String(state.selectedPoolKey)) ? state.selectedPoolKey : keys[0]
      }),
      true,
      'INIT_MARKETS'
    );
  },
  selectMarket: (address: string) => set(state => {
    return {
      ...state,
      selectedMarketKey: address,
    }
  }, false, 'SELECT_MARKET'),
  updateOpenOrders: (update: OpenOrders) => set(state => {
    const key = `${update.authority}:${update.market}`;
    return {
      ...state,
      openOrders: {
        ...state.openOrders,
        [key]: update
      }
    }
  }, true, 'UPDATE_OPEN_ORDERS'),
  updateOpenPositions: (update: OpenPositions) => set(state => {
    const key = `${update.authority}:${update.market}`;
    return {
      ...state,
      openPositions: {
        ...state.openPositions,
        [key]: update
      }
    }
  }, true, 'UPDATE_OPEN_POSITIONS')
});
