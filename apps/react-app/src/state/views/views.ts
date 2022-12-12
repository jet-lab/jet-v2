import { atom } from 'recoil';
import { localStorageEffect } from '../effects/localStorageEffect';

// Controls mobile nav drawer
export const NavDrawerOpen = atom({
  key: 'navDrawerOpen',
  default: false as boolean
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

// Swaps View component order
export const SwapsViewOrder = atom({
  key: 'SwapsViewOrder',
  default: ['accountSnapshot', 'swapsRow', 'fullAccountBalance'] as string[],
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
