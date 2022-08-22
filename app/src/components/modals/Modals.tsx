import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { JupiterProvider } from '@jup-ag/react-hook';
import { Cluster } from '../../state/settings/settings';
import { CurrentAccount } from '../../state/user/accounts';
import { Geobanned } from '../../state/settings/localization/localization';
import {
  WalkthroughModal as WalkthroughModalState,
  WalletModal as WalletModalState,
  NewAccountModal as NewAccountModalState,
  EditAccountModal as EditAccountModalState,
  PairSearchModal as PairSearchModalState,
  SettingsModal as SettingsModalState,
  NotificationsModal as NotificationsModalState
} from '../../state/modals/modals';
import { CurrentAction } from '../../state/actions/actions';
import { useProvider } from '../../utils/jet/provider';
import { GeobannedModal } from './GeobannedModal';
import { DisclaimerModal } from './DisclaimerModal';
import { WalkthroughModal } from './WalkthroughModal';
import { WalletModal } from './WalletModal';
import { DepositWithdrawModal } from './actions/DepositWithdrawModal';
import { BorrowRepayModal } from './actions/BorrowRepayModal';
import { TransferModal } from './actions/TransferModal';
import { SwapModal } from './actions/SwapModal';
import { NewAccountModal } from './NewAccountModal';
import { EditAccountModal } from './EditAccountModal';
import { SettingsModal } from './SettingsModal';
import { NotificationsModal } from './NotificationsModal';
import { PairSearchModal } from '../TradeView/PairSelector/PairSearchModal';
import { LiquidationModal } from './LiquidationModal';

export function Modals(): JSX.Element {
  const { provider } = useProvider();
  const { connection } = provider;
  const { publicKey } = useWallet();
  const cluster = useRecoilValue(Cluster);
  const currentAccount = useRecoilValue(CurrentAccount);
  const geobanned = useRecoilValue(Geobanned);
  const walkthroughModalOpen = useRecoilValue(WalkthroughModalState);
  const WalletModalOpen = useRecoilValue(WalletModalState);
  const currentAction = useRecoilValue(CurrentAction);
  const newAccountModalOpen = useRecoilValue(NewAccountModalState);
  const editAccountModalOpen = useRecoilValue(EditAccountModalState);
  const settingsModalOpen = useRecoilValue(SettingsModalState);
  const notificationsModalOpen = useRecoilValue(NotificationsModalState);
  const pairSearchModalOpen = useRecoilValue(PairSearchModalState);
  const liquidationModalOpen = currentAccount?.isBeingLiquidated;

  // Disable scroll when these modals are open
  useEffect(() => {
    if (
      geobanned ||
      walkthroughModalOpen ||
      WalletModalOpen ||
      currentAction ||
      newAccountModalOpen ||
      editAccountModalOpen ||
      settingsModalOpen ||
      notificationsModalOpen ||
      pairSearchModalOpen ||
      liquidationModalOpen
    ) {
      document.body.style.overflowY = 'hidden';
    } else {
      document.body.style.overflowY = 'unset';
    }
  }, [
    geobanned,
    walkthroughModalOpen,
    WalletModalOpen,
    currentAction,
    newAccountModalOpen,
    editAccountModalOpen,
    settingsModalOpen,
    notificationsModalOpen,
    pairSearchModalOpen,
    liquidationModalOpen
  ]);

  return (
    <>
      <GeobannedModal />
      <DisclaimerModal />
      <WalkthroughModal />
      <WalletModal />
      <DepositWithdrawModal />
      <BorrowRepayModal />
      <JupiterProvider connection={connection} cluster={cluster} userPublicKey={publicKey ?? undefined}>
        <SwapModal />
      </JupiterProvider>
      <TransferModal />
      <NewAccountModal />
      <EditAccountModal />
      <SettingsModal />
      <NotificationsModal />
      <PairSearchModal />
      <LiquidationModal />
    </>
  );
}
