import { atom } from 'recoil';

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
