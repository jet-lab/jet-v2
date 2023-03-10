import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';

// interface WalletToken {
//   address: string;
//   symbol: string;
//   amount: number;
// }

interface Wallet {
  pubkey: string;
  // accounts: Record<string, MarginAccount>;
  // tokens: Record<string, WalletToken>;
}

export interface AccountsSlice {
  accounts: Record<string, Wallet>;
  selectedWallet: string | null;
  connectWallet: (wallet: string) => void;
  disconnectWallet: () => void;
}

export const createAccountsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], AccountsSlice> = (set, get) => ({
  accounts: {},
  selectedWallet: null,
  connectWallet: async wallet => {
    set(() => {
      const cluster = get().settings.cluster
      initWebsocket(cluster, wallet)
      return ({ selectedWallet: wallet })
    }, false, 'CONNECT_WALLET');
  },
  disconnectWallet: () => set(() => ({ selectedWallet: null }), false, 'DISCONNECT_WALLET')
});
