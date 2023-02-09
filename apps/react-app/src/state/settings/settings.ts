import { atom } from 'recoil';
import { localStorageEffect } from '../effects/localStorageEffect';

// Disclaimer accepted
export const DisclaimersAccepted = atom({
  key: 'disclaimersAccepted',
  default: {} as Record<string, boolean>,
  effects: [localStorageEffect('jetAppDisclaimerAccepted')]
});

// Fiat Currency
export const fiatOptions: Record<string, string> = {
  USD: '$',
  ARS: '',
  AUD: 'A$',
  CAD: 'CA$',
  CHF: '',
  CNH: '',
  EUR: '€',
  GBP: '£',
  HKD: 'HK$',
  IDR: '',
  INR: '₹',
  JPY: '¥',
  KRW: '₩',
  NGN: '',
  NZD: 'NZ$',
  SGD: '',
  VND: '₫',
  ZAR: ''
};
export const FiatCurrency = atom({
  key: 'fiatCurrency',
  default: 'USD' as string,
  effects: [localStorageEffect('jetAppFiatCurrency')]
});
export const FiatValues = atom({
  key: 'fiatValues',
  default: true as boolean,
  effects: [localStorageEffect('jetAppFiatValues')]
});
export const USDConversionRates = atom({
  key: 'usdConversionRates',
  default: {} as Record<string, number>
});

// Block Explorer
export type Explorer = 'solanaExplorer' | 'solscan' | 'solanaBeach';
export const blockExplorers: Record<Explorer, Record<string, string>> = {
  solanaExplorer: {
    name: 'Solana Explorer',
    img: 'img/explorers/solana_explorer.svg',
    url: 'https://explorer.solana.com/tx/'
  },
  solscan: {
    name: 'Solscan',
    img: 'img/explorers/solscan.svg',
    url: 'https://solscan.io/tx/'
  },
  solanaBeach: {
    name: 'Solana Beach',
    img: 'img/explorers/solana_beach.svg',
    url: 'https://solanabeach.io/transaction/'
  }
};

// Unix / Local Time
export type TimeDisplay = 'local' | 'utc';
export const timeDisplayOptions: TimeDisplay[] = ['local', 'utc'];
export const PreferredTimeDisplay = atom({
  key: 'preferredTimeDisplay',
  default: 'local' as TimeDisplay,
  effects: [localStorageEffect('jetAppPreferredTimeDisplay')]
});
export const PreferDayMonthYear = atom({
  key: 'preferDayMonthYear',
  default: true as boolean,
  effects: [localStorageEffect('jetAppPreferDayMonthYear')]
});
