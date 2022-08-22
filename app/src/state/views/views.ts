import { useLocation } from 'react-router-dom';
import { atom } from 'recoil';
import { localStorageEffect } from '../effects/localStorageEffect';

// Current path (for navbar active tab styling)
export const CurrentPath = atom({
  key: 'currentPath',
  default: '/' as string,
  effects: [
    ({ setSelf }) => {
      const { pathname } = useLocation();
      setSelf(pathname);
    }
  ]
});

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
  default: ['accountSnapshot', 'poolsTable', 'poolsRow'] as string[],
  effects: [localStorageEffect('jetAppPoolsViewOrder')],
  dangerouslyAllowMutability: true
});
export const PoolsRowOrder = atom({
  key: 'PoolsRowOrder',
  default: ['poolDetail', 'radar'] as string[],
  effects: [localStorageEffect('jetAppPoolsRowOrder')],
  dangerouslyAllowMutability: true
});

// Accounts View component order
export const AccountsViewOrder = atom({
  key: 'accountsViewOrder',
  default: ['accountSnapshot', 'fullAccountHistory', 'fullAccountBalance'] as string[],
  effects: [localStorageEffect('jetAppAccountsViewOrder')],
  dangerouslyAllowMutability: true
});
