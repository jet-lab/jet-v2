export interface OpenOrders {
  market: string;
  authority: string;
  open_orders: OpenOrder[];
  unfilled_borrow: number;
  unfilled_lend: number;
}

export interface OpenOrder {
  order_tag: string;
  order_id: string;
  is_lend_order: boolean;
  is_cancelling: boolean;
  total_quote_qty: number;
  filled_quote_qty: number;
  is_auto_stake: boolean;
  created_timestamp: number;
  rate: number;
}

export interface OpenPositions {
  market: string;
  authority: string;
  loans: Loan[];
  deposits: Deposit[];
  total_borrowed: number;
  total_lent: number;
}

export interface Loan {
  id: number;
  address: string;
  sequence_number: number;
  maturation_timestamp: number;
  balance: number;
  is_marked_due: boolean;
  created_timestamp: number;
  payer: string
  rate: number;
}

export interface Deposit {
  id: number
  address: string,
  sequence_number: number,
  maturation_timestamp: number,
  balance: number,
  rate: number,
  payer: string,
  created_timestamp: number
}

export type Order = {
  owner: any,
  order_tag: any,
  base_size: bigint,
  price: bigint,
}

export type OrderbookSnapshot = {
  bids: Array<Order>,
  asks: Array<Order>,
};

export interface SwapLiquidity {
  base: string;
  quote: string;
  bids: number[][];
  asks: number[][];
  liquidity_range: number[];
  price_range: number[];
}