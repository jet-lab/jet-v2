interface TOKEN_PRICE_UPDATE {
  underlying_token_mint: string;
  adapter?: string;
  mint: string;
  price: {
    id: string;
    price: {
      price: string;
      conf: string;
      expo: number;
      publish_time: number;
    };
    ema_price: {
      price: string;
      conf: string;
      expo: number;
      publish_time: number;
    };
  };
}

interface PRICE_UPDATE {
  type: 'PRICE-UPDATE';
  payload: Record<string, TOKEN_PRICE_UPDATE>;
}

interface MARGIN_POOL_UPDATE {
  type: 'MARGIN-POOL-UPDATE';
  payload: {
    address: string;
    borrowed_tokens: number[];
    uncollected_fees: number[];
    deposit_tokens: number;
    deposit_notes: number;
    loan_notes: 0;
    accrued_until: number;
  };
}

// *** END MARGIN POOL UPDATE EVENT ***

// *** SUBSCRIBE EVENT ***
interface SUBSCRIBE {
  type: 'SUBSCRIBE';
  payload: {
    wallet: string;
    margin_accounts: string[];
  };
}
// *** END SUBSCRIBE EVENT ***

type APPLICATION_WS_EVENTS = SUBSCRIBE;

type JET_WS_EVENTS = PRICE_UPDATE | MARGIN_POOL_UPDATE;
