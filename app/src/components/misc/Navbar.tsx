import { useState } from 'react';
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { useLocation, useNavigate } from 'react-router-dom';
import { useWallet } from '@solana/wallet-adapter-react';
import { CurrentPath } from '../../state/views/views';
import { Dictionary } from '../../state/settings/localization/localization';
import { WalletModal } from '../../state/modals/modals';
import { NotificationsModal, SettingsModal } from '../../state/modals/modals';
import { animateViewOut } from '../../utils/ui';
import { Typography } from 'antd';
import { TpsBanner } from './TpsBanner';
import { WalletButton } from './WalletButton';
import { SettingFilled } from '@ant-design/icons';
import { ReactComponent as NotificationsBell } from '../../styles/icons/notifications-bell.svg';

export function Navbar(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { pathname } = useLocation();
  const navigate = useNavigate();
  const [currentPath, setCurrentPath] = useRecoilState(CurrentPath);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const { connected } = useWallet();
  const setWalletModalOpen = useSetRecoilState(WalletModal);
  const [settingsModalOpen, setSettingsModalOpen] = useRecoilState(SettingsModal);
  const [notificationsModalOpen, setNotificationsModalOpen] = useRecoilState(NotificationsModal);
  const navLinks = [
    { title: dictionary.tradeView.title, route: '/' },
    { title: dictionary.poolsView.title, route: '/pools' },
    { title: dictionary.accountsView.title, route: '/accounts' }
  ];
  const mobileFooterLinks = [
    { title: dictionary.termsPrivacy.termsOfService, url: 'https://www.jetprotocol.io/legal/terms-of-service' },
    { title: dictionary.termsPrivacy.privacyPolicy, url: 'https://www.jetprotocol.io/legal/privacy-policy' },
    { title: dictionary.termsPrivacy.glossary, url: 'https://docs.jetprotocol.io/jet-protocol/terms-and-definitions' }
  ];
  const { Text } = Typography;

  // Change view with a transition
  function changeView(route: string, replace = true) {
    if (route === currentPath) {
      return;
    }

    setCurrentPath(route);
    setTimeout(() => navigate(route, { replace }), 500);
    animateViewOut();
  }

  return (
    <div className={`navbar-container flex-centered column ${drawerOpen ? 'drawer-open' : ''}`}>
      <TpsBanner />
      {/* Desktop Nav */}
      <nav className="desktop flex align-center justify-between">
        <div className="nav-section flex align-center justify-start">
          <img
            onClick={() => changeView('/')}
            className="nav-logo"
            src="img/jet/jet_logo.png"
            width="100%"
            height="auto"
            alt="Jet Protocol"
          />
        </div>
        <div className="navbar nav-section flex-centered">
          {navLinks.map(link => (
            <Text
              key={link.title}
              onClick={() => changeView(link.route)}
              className={`nav-link ${currentPath === link.route ? 'active' : ''}`}>
              {link.title}
            </Text>
          ))}
        </div>
        <div className="nav-section flex align-center justify-end">
          <div className="flex-centered">
            <NotificationsBell
              className={`notifications-btn icon-btn ${notificationsModalOpen ? 'active' : ''}`}
              onClick={() =>
                connected ? setNotificationsModalOpen(!notificationsModalOpen) : setWalletModalOpen(true)
              }
            />
            <SettingFilled
              className={`settings-btn icon-btn ${settingsModalOpen ? 'active' : ''}`}
              onClick={() => setSettingsModalOpen(!settingsModalOpen)}
            />
          </div>
          <WalletButton />
        </div>
      </nav>
      {/* Mobile Nav */}
      <nav className="mobile flex align-center justify-between">
        <div className="flex-centered">
          <NotificationsBell
            className={`notifications-btn icon-btn ${notificationsModalOpen ? 'active' : ''}`}
            onClick={() => (connected ? setNotificationsModalOpen(!notificationsModalOpen) : setWalletModalOpen(true))}
          />
          <SettingFilled
            className={`settings-btn icon-btn ${settingsModalOpen ? 'active' : ''}`}
            onClick={() => {
              setSettingsModalOpen(true);
              setDrawerOpen(false);
            }}
          />
        </div>
        <img
          onClick={() => changeView('/')}
          className="nav-logo"
          src="img/jet/jet_logo.png"
          width="100%"
          height="auto"
          alt="Jet Protocol"
        />
        <div
          className={`hamburger flex align-center justify-between column ${drawerOpen ? 'close' : ''}`}
          onClick={() => setDrawerOpen(!drawerOpen)}>
          <span></span>
          <span></span>
          <span></span>
        </div>
        <div className="drawer flex align-center justify-between column">
          <div className="drawer-top flex-centered column">
            {navLinks.map(link => (
              <Text
                key={link.title}
                onClick={() => {
                  changeView(link.route);
                  setDrawerOpen(false);
                }}
                className={`nav-link ${pathname === link.route ? 'active' : ''}`}>
                {link.title}
              </Text>
            ))}
            <WalletButton mobile />
          </div>
          <div className="drawer-bottom flex-centered column">
            {mobileFooterLinks.map(link => (
              <a key={link.title} href={link.url} className="footer-link" rel="noopener noreferrer" target="_blank">
                {link.title}
              </a>
            ))}
          </div>
        </div>
      </nav>
    </div>
  );
}
