import { StateCreator } from 'zustand';

export const createOrderBookSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], OrderBookSlice> = set => ({
  orderbooks: {},
  updateOrderBook: msg =>
    set(
      state => {
        return {
          ...state,
          orderbooks: {
            ...state.orderbooks,
            [msg.payload.market]: {
              market: msg.payload.market,
              asks: msg.payload.asks,
              bids: msg.payload.bids
            }
          }
        };
      },
      false,
      'UPDATE_ORDERBOOK'
    )
});
