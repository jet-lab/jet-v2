import { atom } from 'recoil';
import axios from 'axios';
import { localStorageEffect } from '../effects/localStorageEffect';
import { getPing } from '../../utils/ui';

// Disclaimer accepted
export const DisclaimersAccepted = atom({
  key: 'disclaimersAccepted',
  default: {} as Record<string, boolean>,
  effects: [localStorageEffect('jetAppDisclaimerAccepted')]
});

// RPC Input
export interface Node {
  name: string;
  devnet: string;
  devnetPing: number;
  mainnetBeta: string;
  mainnetBetaPing: number;
}
export type NodeOption = 'default' | 'solana' | 'projectSerum' | 'custom';
export const rpcNodeOptions: NodeOption[] = ['default', 'solana', 'projectSerum', 'custom'];
export const rpcNodes: Record<NodeOption, Node> = {
  default: {
    name: 'Default',
    devnet: `https://jetprot-develope-26c4.devnet.rpcpool.com/${process.env.REACT_APP_RPC_DEV_TOKEN ?? ''}`,
    devnetPing: 0,
    mainnetBeta: `https://jetprot-main-0d7b.mainnet.rpcpool.com/${process.env.REACT_APP_RPC_TOKEN ?? ''}`,
    mainnetBetaPing: 0
  },
  solana: {
    name: 'Solana',
    devnet: 'https://api.devnet.solana.com',
    devnetPing: 0,
    mainnetBeta: 'https://api.mainnet-beta.solana.com/',
    mainnetBetaPing: 0
  },
  projectSerum: {
    name: 'Project Serum',
    devnet: 'https://solana-api.projectserum.com/',
    devnetPing: 0,
    mainnetBeta: 'https://solana-api.projectserum.com/',
    mainnetBetaPing: 0
  },
  custom: {
    name: 'Custom',
    devnet: '',
    devnetPing: 0,
    mainnetBeta: '',
    mainnetBetaPing: 0
  }
};
export const RpcNodes = atom({
  key: 'rpcNodes',
  default: rpcNodes as Record<NodeOption, Node>,
  effects: [
    ({ setSelf }) => {
      const customMainnetNode = localStorage.getItem('jetCustomNode-mainnet');
      const customDevnetNode = localStorage.getItem('jetCustomNode-devnet');
      if (customMainnetNode) {
        rpcNodes.custom.mainnetBeta = customMainnetNode;
      }
      if (customDevnetNode) {
        rpcNodes.custom.devnet = customDevnetNode;
      }
      for (const nodeOption in rpcNodes) {
        const node = rpcNodes[nodeOption as NodeOption];
        getPing(node.devnet).then((ping: number) => (node.devnetPing = ping));
        getPing(node.mainnetBeta).then((ping: number) => (node.mainnetBetaPing = ping));
      }
      setSelf(rpcNodes);
    }
  ],
  dangerouslyAllowMutability: true
});
export const PreferredRpcNode = atom({
  key: 'preferredRpcNode',
  default: 'default' as NodeOption,
  effects: [localStorageEffect('jetAppPreferredNode')]
});

// Connection cluster
export type ClusterOption = 'devnet' | 'mainnet-beta';
export const clusterOptions: ClusterOption[] = ['devnet', 'mainnet-beta'];
export const Cluster = atom({
  key: 'cluster',
  default: 'mainnet-beta' as ClusterOption,
  effects: [localStorageEffect('jetAppCluster')]
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
  default: {} as Record<string, number>,
  effects: [
    ({ setSelf }) => {
      axios
        .get('https://api.jetprotocol.io/v1/rates')
        .then(resp => {
          const conversions = resp.data;
          if (conversions) {
            setSelf(conversions.rates);
          }
        })
        .catch(err => err);
    }
  ]
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
export const BlockExplorer = atom({
  key: 'blockExplorer',
  default: 'solscan' as Explorer,
  effects: [localStorageEffect('jetAppPreferredExplorer')]
});

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

// Light / Dark Theme
export const LightTheme = atom({
  key: 'lightTheme',
  default: false as boolean,
  effects: [localStorageEffect('jetAppLightTheme')]
});
