export interface FixedOrder {
  tag: string;
  details: FixedOrderDetails;
  fills: FixedOrderFill[];
}

export interface FixedOrderDetails {
  id: number;
  order_tag: string;
  order_id?: string;
  market: string;
  authority: string;
  margin_user: string;
  order_type: string;
  order_status: string;
  total_base_qty: number;
  total_quote_qty: number;
  total_base_qty_posted: number;
  limit_price: number;
  rate: number;
  is_auto_stake: boolean;
  is_post_only: boolean;
  is_post_allowed: boolean;
  created_slot: number;
  created_timestamp: Date;
  modified_slot: number;
  modified_timestamp: Date;
}

export interface FixedOrderFill {
  id: number;
  order_tag: string;
  sequence_number: number;
  market: string;
  authority: string;
  fill_side: string;
  fill_status: string;
  quote_filled: number;
  base_filled: number;
  maturation_timestamp: Date;
  fill_slot: number;
  fill_timestamp: Date;
  modified_slot: number;
  modified_timestamp: Date;
}
