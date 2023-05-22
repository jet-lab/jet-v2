import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';

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
