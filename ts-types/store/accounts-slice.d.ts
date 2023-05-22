// interface WalletToken {
//   address: string;
//   symbol: string;
//   amount: number;
// }

interface MarginAccountData {
  address: string;
  owner: string;
  liquidator: string;
  positions: MarginPosition[];
}

interface MarginPosition {
  adapter: string;
  address: string;
  balance: number;
  balanceTimestamp: number;
  exponent: number;
  kind: 'Collateral' | 'AdapterCollateral' | 'Claim';
  maxStaleness: number;
  price: {
    exponent: number;
    isValid: number;
    timestamp: number;
    value: number;
  };
  token: string;
  value: string; // Number192 formatted as decimal string
  valueModifier: number;
}

interface Wallet {
  pubkey: string;
  accounts: Record<string, MarginAccountData>;
  selectedMarginAccount: string | null;
  lookupTables: Record<string, string[]>;
  // tokens: Record<string, WalletToken>;
}

interface LookupTable {
  address: string;
  // Avoid storing a class, and having to import the lookup table types
  data: Uint8Array;
}

interface AccountsSlice {
  wallets: Record<string, Wallet>;
  selectedWallet: string | null;
  connectWallet: (wallet: string) => void;
  disconnectWallet: () => void;
  // The lookup addresses of the airspace. Only storing here as we don't yet have an airspace slice
  airspaceLookupTables: LookupTable[];
  updateLookupTables: (tables: LookupTable[]) => void;
  updateMarginAccount: (update: MarginAccountData) => void;
  initAllMarginAccounts: (update: Record<string, MarginAccountData>) => void;
  selectMarginAccount: (address: string) => void;
}
