import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';

type Explorer = 'solanaExplorer' | 'solscan' | 'solanaBeach';

export type Cluster = 'localnet' | 'devnet' | 'mainnet-beta';
export type NodeOption = 'default' | 'custom';

interface RPC {
  name: string;
  devnet: string;
  localnet: string;
  'mainnet-beta': string;
  pings: Record<Cluster, number>;
}

export const rpc = {
  name: 'Default',
  devnet: `https://jetprot-develope-26c4.devnet.rpcpool.com/${process.env.REACT_APP_RPC_DEV_TOKEN ?? ''}`,
  'mainnet-beta': `https://jetprot-main-0d7b.mainnet.rpcpool.com/${process.env.REACT_APP_RPC_TOKEN ?? ''}`,
  localnet: 'http://localhost:8899',
  pings: {
    'mainnet-beta': 0,
    devnet: 0,
    localnet: 0
  }
};

interface Settings {
  cluster: Cluster;
  explorer: Explorer;
  rpc: RPC;
}
export interface SettingsSlice {
  settings: Settings;
  updateSettings: (payload: Partial<Settings>) => void;
}
export const createSettingsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], SettingsSlice> = (
  set,
  get
) => ({
  settings: {
    cluster: 'mainnet-beta',
    explorer: 'solanaExplorer',
    rpc: rpc
  },
  updateSettings: (payload: Partial<Settings>) => {
    if (payload.cluster !== get().settings.cluster) {
      initWebsocket(payload.cluster);
    }
    return set(state => ({ settings: { ...state.settings, ...payload } }), false, 'UPDATE_SETTINGS');
  }
});
