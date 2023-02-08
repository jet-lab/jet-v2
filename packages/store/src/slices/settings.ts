import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';

type Explorer = 'solanaExplorer' | 'solscan' | 'solanaBeach';
export type Cluster = 'localnet' | 'devnet' | 'mainnet-beta';

interface Settings {
  cluster: Cluster;
  explorer: Explorer;
}
export interface SettingsSlice {
  settings: Settings;
  updateSettings: (payload: Settings) => void;
}
export const createSettingsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], SettingsSlice> = (
  set,
  get
) => ({
  settings: {
    cluster: 'mainnet-beta',
    explorer: 'solanaExplorer'
  },
  updateSettings: (payload: Partial<Settings>) => {
    if (payload.cluster !== get().settings.cluster) {
      initWebsocket(payload.cluster);
    }
    return set(state => ({ settings: { ...state.settings, ...payload } }), false, 'UPDATE_SETTINGS');
  }
});
