import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';

const rpc = {
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

export const createSettingsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], SettingsSlice> = (
  set,
  get
) => ({
  settings: {
    cluster: window.location.href.includes('devnet')
      ? 'devnet'
      : window.location.href.includes('cluster=localnet')
      ? 'localnet'
      : 'mainnet-beta',
    explorer: 'solanaExplorer',
    rpc: rpc
  },
  updateSettings: (payload: Partial<Settings>) => {
    if (payload.cluster !== get().settings.cluster) {
      initWebsocket(payload.cluster, get().selectedWallet);
    }
    return set(state => ({ settings: { ...state.settings, ...payload } }), false, 'UPDATE_SETTINGS');
  }
});
