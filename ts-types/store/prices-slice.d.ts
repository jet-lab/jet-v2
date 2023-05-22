interface PricesSlice {
  prices?: Record<string, PriceInfo>;
  updatePrices: (update: PRICE_UPDATE) => void;
}
