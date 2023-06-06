// interface WalletToken {
//   address: string;
//   symbol: string;
//   amount: number;
// }

interface Wallet {
  pubkey: string;
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
  // Margin accounts
  marginAccounts: Record<string, MarginAccountData>;
  selectedMarginAccount: string | null;
  updateMarginAccount: (update: MarginAccountData) => void;
  initAllMarginAccounts: (update: Record<string, MarginAccountData>) => void;
  selectMarginAccount: (address: string) => void;
  // The lookup addresses of margin accounts.
  marginAccountLookupTables: Record<string, LookupTable[]>;
  updateMarginAccountLookupTables: (address: string, tables: LookupTable[]) => void;
  // The lookup addresses of the airspace. Only storing here as we don't yet have an airspace slice
  airspaceLookupTables: LookupTable[];
  updateAirspaceLookupTables: (tables: LookupTable[]) => void;
}
