import { atom } from 'recoil';
import { localStorageEffect } from '../effects/localStorageEffect';

// Trade View component order
export const TradeViewOrder = atom({
  key: 'tradeViewOrder',
  default: ['accountSnapshot', 'pairSelector', 'tradeRow', 'candleStickChart', 'pairRelatedAccount'] as string[],
  effects: [localStorageEffect('jetAppTradeViewOrder')],
  dangerouslyAllowMutability: true
});
export const TradeRowOrder = atom({
  key: 'tradeRowOrder',
  default: ['orderEntry', 'orderbook', 'recentTrades'] as string[],
  effects: [localStorageEffect('jetAppTradeRowOrder')],
  dangerouslyAllowMutability: true
});

// Pools View component order
export const PoolsViewOrder = atom({
  key: 'PoolsViewOrder',
  default: ['accountSnapshot', 'poolsRow', 'poolsTable'] as string[],
  effects: [localStorageEffect('jetAppPoolsViewOrder')],
  dangerouslyAllowMutability: true
});
export const PoolsRowOrder = atom({
  key: 'PoolsRowOrder',
  default: ['poolDetail', 'radar'] as string[],
  effects: [localStorageEffect('jetAppPoolsRowOrder')],
  dangerouslyAllowMutability: true
});

// Swaps View component order
export const SwapsViewOrder = atom({
  key: 'SwapsViewOrder',
  default: ['accountSnapshot', 'swapsRow', 'swapsHistory'] as string[],
  effects: [localStorageEffect('jetAppSwapsViewOrder')],
  dangerouslyAllowMutability: true
});
export const SwapsRowOrder = atom({
  key: 'SwapsRowOrder',
  default: ['swapEntry', 'swapsGraph'] as string[],
  effects: [localStorageEffect('jetAppSwapsRowOrder')],
  dangerouslyAllowMutability: true
});

// Accounts View component order
export const AccountsViewOrder = atom({
  key: 'accountsViewOrder',
  default: ['accountSnapshot', 'fullAccountHistory', 'fullAccountBalance'] as string[],
  effects: [localStorageEffect('jetAppAccountsViewOrder')],
  dangerouslyAllowMutability: true
});
