import { atom } from 'recoil';

export type NetworkState = 'connected' | 'loading' | 'error';

export const NetworkStateAtom = atom<NetworkState>({
  key: 'network-unavailable',
  default: 'loading'
});
