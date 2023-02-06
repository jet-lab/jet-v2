import { StateCreator } from 'zustand';
import { JetStore } from '../store';

export interface SettingsSlice {
  settings: {
    cluster: 'localnet' | 'devnet' | 'mainnet-beta';
  };
  updateSetting: (setting: string, value: string) => void;
}
export const createSettingsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], SettingsSlice> = set => ({
  settings: {
    cluster: 'mainnet-beta'
  },
  updateSetting: (setting: string, value: string) =>
    set(state => ({ settings: { ...state.settings, [setting]: value } }), false, 'UPDATE_SETTINGS')
});
