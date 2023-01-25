export interface PriceDetails {
  price: number,
  conf: number,
  expo: number,
  publish_time: Date
}

export interface PricesSlice {
  prices: Record<string, {
    price: PriceDetails,
    ema_price: PriceDetails
  }>
}