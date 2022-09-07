import { atom } from 'recoil';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { TokenAmount, OrderSide as MarginOrderSide, OrderType as MarginOrderType } from '@jet-lab/margin';

// Order Side
export const OrderSide = atom({
  key: 'orderSide',
  default: 'buy' as MarginOrderSide
});

// Order Type
export const orderTypes: MarginOrderType[] = ['limit', 'ioc', 'postOnly'];
export const OrderType = atom({
  key: 'orderType',
  default: 'limit' as MarginOrderType
});

// Order Price
export const OrderPrice = atom({
  key: 'orderPrice',
  default: 0 as number
});
export const OrderPriceString = atom({
  key: 'orderPriceString',
  default: undefined as string | undefined
});

// Order Size
export const OrderSize = atom({
  key: 'orderSize',
  default: TokenAmount.zero(Math.log10(LAMPORTS_PER_SOL)) as TokenAmount
});
export const OrderSizeString = atom({
  key: 'orderSizeString',
  default: undefined as string | undefined
});

// Order Amount
export const OrderAmount = atom({
  key: 'orderAmount',
  default: TokenAmount.zero(Math.log10(LAMPORTS_PER_SOL)) as TokenAmount
});
export const OrderAmountString = atom({
  key: 'orderAmountString',
  default: undefined as string | undefined
});
