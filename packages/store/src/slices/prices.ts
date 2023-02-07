import { StateCreator } from 'zustand';
import { PRICE_UPDATE, TOKEN_PRICE_UPDATE } from '../events';
import { JetStore } from '../store';

export interface PricesSlice {
  prices?: Record<
    string,
    {
      price: number;
      ema: number;
      confidence: number;
      timestamp: Date;
    }
  >;
  updatePrices: (update: PRICE_UPDATE) => void;
}

export const createPricesSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], PricesSlice> = set => ({
  updatePrices: (update: PRICE_UPDATE) =>
    set(
      () => {
        const prices = Object.entries(update.payload).reduce(
          (acc, [token_mint, data]: [string, TOKEN_PRICE_UPDATE]) => {
            if (data.price) {
              acc[token_mint] = {
                price: parseInt(data.price.price) * 10 ** data.price.expo,
                ema: parseInt(data.ema_price.price) * 10 ** data.price.expo,
                confidence: parseInt(data.price.conf) * 10 ** data.price.expo,
                timestamp: new Date(data.price.publish_time * 1000)
              };
            } else {
              console.log(data);
            }
            return acc;
          },
          {}
        );
        return { prices };
      },
      false,
      'UPDATE_PRICE'
    )
});
