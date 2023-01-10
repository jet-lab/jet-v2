export interface OpenOrders {
  market:          string;
  authority:       string;
  open_orders:     OpenOrder[];
  unfilled_borrow: number;
  unfilled_lend:   number;
}

export interface OpenOrder {
  order_tag:         string;
  order_id:          string;
  is_lend_order:     boolean;
  is_cancelling:     boolean;
  total_quote_qty:   number;
  filled_quote_qty:  number;
  is_auto_stake:     boolean;
  created_timestamp: number;
}

export interface OpenPositions {
  market:         string;
  authority:      string;
  loans:          Loan[];
  total_borrowed: number;
  deposits:       any[];
  total_lent:     number;
}

export interface Loan {
  id:                   number;
  address:              string;
  sequence_number:      number;
  maturation_timestamp: number;
  balance:              number;
  is_marked_due:        boolean;
  created_timestamp:    number;
  payer: string
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
