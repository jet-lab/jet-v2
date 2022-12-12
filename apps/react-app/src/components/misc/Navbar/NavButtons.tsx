import { useRecoilState } from 'recoil';
import { SettingsModal } from '@state/modals/modals';
import { WalletButton } from '../WalletButton';
import { SettingFilled } from '@ant-design/icons';
// import { ReactComponent as NotificationsBell } from '@assets/icons/notifications-bell.svg';

// Modal buttons for Navbar (settings, notifications and optionally the wallet button)
export function NavButtons(props: { showWalletButton?: boolean }): JSX.Element {
  // const { connected } = useWallet();
  // const setWalletModalOpen = useSetRecoilState(WalletModal);
  const [settingsModalOpen, setSettingsModalOpen] = useRecoilState(SettingsModal);
  // const [notificationsModalOpen, setNotificationsModalOpen] = useRecoilState(NotificationsModal);

  return (
    <>
      <div className="flex-centered">
        {/* 
        <NotificationsBell
          className={`notifications-btn icon-btn ${notificationsModalOpen ? 'active' : ''}`}
          onClick={() => (connected ? setNotificationsModalOpen(!notificationsModalOpen) : setWalletModalOpen(true))}
        /> 
        */}
        <SettingFilled
          className={`settings-btn icon-btn ${settingsModalOpen ? 'active' : ''}`}
          onClick={() => setSettingsModalOpen(!settingsModalOpen)}
        />
      </div>
      {props.showWalletButton ? <WalletButton /> : <></>}
    </>
  );
}
