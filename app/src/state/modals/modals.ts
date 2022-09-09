import { atom } from 'recoil';
import { localStorageEffect } from '../effects/localStorageEffect';

// Connect wallet modal
export const WalletModal = atom({
  key: 'connectModal',
  default: false as boolean
});

// New account modal
export const NewAccountModal = atom({
  key: 'newAccountModal',
  default: false as boolean
});
// Edit account modal
export const EditAccountModal = atom({
  key: 'editAccountModal',
  default: false as boolean
});

// Settings modal
export const SettingsModal = atom({
  key: 'settingsModal',
  default: false as boolean
});

// Notifications modal
export const NotificationsModal = atom({
  key: 'notificationsModal',
  default: false as boolean
});

// Pair search modal
export const PairSearchModal = atom({
  key: 'pairSearchModal',
  default: false as boolean
});

// Product walkthrough
export const WalkthroughModal = atom({
  key: 'walkthroughModal',
  default: false as boolean
});
export const WalkthroughCompleted = atom({
  key: 'walkthroughCompleted',
  default: false as boolean,
  effects: [localStorageEffect('jetAppWalkthroughCompleted')]
});
