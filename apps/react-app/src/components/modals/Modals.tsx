import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import {
  WalletModal as WalletModalState,
  NewAccountModal as NewAccountModalState,
  SettingsModal as SettingsModalState,
  NotificationsModal as NotificationsModalState,
  AccountUpgradeModal as AccountUpgradeModalState
} from '@state/modals/modals';
import { CurrentAction } from '@state/actions/actions';
import { DisclaimerModal } from './DisclaimerModal';
import { WalletModal } from './WalletModal';
import { DepositWithdrawModal } from './actions/DepositWithdrawModal';
import { BorrowRepayModal } from './actions/BorrowRepayModal';
import { TransferModal } from './actions/TransferModal';
import { NewAccountModal } from './NewAccountModal';
import { SettingsModal } from './SettingsModal';
import { NotificationsModal } from './NotificationsModal';
import { AccountUpgradeModal } from './AccountUpgradeModal';

// Wrapper component to include all app modals
export function Modals(): JSX.Element {
  const WalletModalOpen = useRecoilValue(WalletModalState);
  const currentAction = useRecoilValue(CurrentAction);
  const newAccountModalOpen = useRecoilValue(NewAccountModalState);
  const settingsModalOpen = useRecoilValue(SettingsModalState);
  const notificationsModalOpen = useRecoilValue(NotificationsModalState);
  const accountUpgradeModalOpen = useRecoilValue(AccountUpgradeModalState);

  // Disable scroll when these modals are open
  useEffect(() => {
    if (WalletModalOpen || currentAction || newAccountModalOpen || settingsModalOpen || notificationsModalOpen || accountUpgradeModalOpen) {
      document.body.style.overflowY = 'hidden';
    } else {
      document.body.style.overflowY = 'unset';
    }
  }, [WalletModalOpen, currentAction, newAccountModalOpen, settingsModalOpen, notificationsModalOpen, accountUpgradeModalOpen]);

  return (
    <>
      <DisclaimerModal />
      <WalletModal />
      <DepositWithdrawModal />
      <BorrowRepayModal />
      <TransferModal />
      <NewAccountModal />
      <SettingsModal />
      <NotificationsModal />
      <AccountUpgradeModal />
    </>
  );
}
