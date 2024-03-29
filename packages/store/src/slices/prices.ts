import { StateCreator } from 'zustand';

export const createPricesSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], PricesSlice> = set => ({
  updatePrices: (update: PRICE_UPDATE) =>
    set(
      () => {
        const prices = Object.entries(update.payload).reduce(
          (acc, [token_mint, data]: [string, TOKEN_PRICE_UPDATE]) => {
            if (data.price) {
              acc[token_mint] = {
                price: +data.price.price.price * 10 ** data.price.price.expo,
                ema: +data.price.ema_price.price * 10 ** data.price.ema_price.expo,
                confidence: +data.price.price.conf * 10 ** data.price.price.expo,
                timestamp: new Date(data.price.price.publish_time * 1000)
              };
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
