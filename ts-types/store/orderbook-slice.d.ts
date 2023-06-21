interface OrderBookSlice {
  orderbooks: Record<string, OrderBook>;
  updateOrderBook: (msg: ORDER_BOOK_SNAPSHOT) => void;
}
