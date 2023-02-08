import { StateCreator } from 'zustand';
import { JetStore } from '../store';

type Explorer = 'solanaExplorer' | 'solscan' | 'solanaBeach';
type Cluster = 'localnet' | 'devnet' | 'mainnet-beta';

interface Settings {
  cluster: Cluster;
  explorer: Explorer;
}
export interface SettingsSlice {
  settings: Settings;
  updateSettings: (payload: Settings) => void;
}
export const createSettingsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], SettingsSlice> = set => ({
  settings: {
    cluster: 'mainnet-beta',
    explorer: 'solanaExplorer'
  },
  updateSettings: (payload: Partial<Settings>) =>
    set(state => ({ settings: { ...state.settings, ...payload } }), false, 'UPDATE_SETTINGS')
});
