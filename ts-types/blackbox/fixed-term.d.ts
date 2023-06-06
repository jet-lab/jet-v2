interface OpenOrders {
  market: string;
  authority: string;
  open_orders: OpenOrder[];
  unfilled_borrow: number;
  unfilled_lend: number;
}

interface OpenOrder {
  order_tag: string;
  order_id: string;
  is_lend_order: boolean;
  is_cancelling: boolean;
  total_quote_qty: number;
  filled_quote_qty: number;
  is_auto_stake: boolean;
  is_auto_roll: boolean;
  created_timestamp: number;
  rate: number;
}

interface OpenPositions {
  market: string;
  authority: string;
  loans: Loan[];
  deposits: Deposit[];
  total_borrowed: number;
  total_lent: number;
}

interface Loan {
  id: number;
  address: string;
  sequence_number: number;
  maturation_timestamp: number;
  principal: number;
  interest: number;
  remaining_balance: number;
  is_marked_due: boolean;
  is_auto_roll: boolean;
  created_timestamp: number;
  payer: string;
  rate: number;
}

interface Deposit {
  id: number;
  address: string;
  sequence_number: number;
  maturation_timestamp: number;
  principal: number;
  interest: number;
  rate: number;
  payer: string;
  is_auto_roll: boolean;
  created_timestamp: number;
}

type Order = {
  owner: any;
  order_tag: any;
  base_size: bigint;
  price: bigint;
};

type OrderbookSnapshot = {
  bids: Array<Order>;
  asks: Array<Order>;
};

type PriceLevel = [price: number, amt: number];
interface SwapLiquidityTokenInfo {
  mint: string;
  symbol: string;
  expo: number;
}
interface SwapLiquidity {
  base: SwapLiquidityTokenInfo;
  quote: SwapLiquidityTokenInfo;
  bids: PriceLevel[];
  asks: PriceLevel[];
  liquidity_range: [min: number, max: number];
  price_range: [min: number, max: number];
}

type FixedTermAccountingShim = Record<string, number>;
