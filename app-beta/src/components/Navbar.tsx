import { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useWallet } from '@solana/wallet-adapter-react';
import { useConnectWalletModal } from '../contexts/connectWalletModal';
import { useSettingsModal } from '../contexts/settingsModal';
import { useLanguage } from '../contexts/localization/localization';
import { shortenPubkey } from '../utils/utils';
import { Button } from 'antd';
import { ReactComponent as WalletIcon } from '../styles/icons/wallet_icon.svg';
import { SettingFilled } from '@ant-design/icons';
export function Navbar(): JSX.Element {
  const { dictionary } = useLanguage();
  const { pathname } = useLocation();
  const { connected, disconnect, publicKey } = useWallet();
  const { setConnecting } = useConnectWalletModal();
  const { setOpen } = useSettingsModal();
  const [drawerOpened, setDrawerOpened] = useState(false);
  const navLinks = [
    { title: dictionary.cockpit.title, route: '/' },
    { title: dictionary.transactions.title, route: '/transactions' }
  ];
  const mobileFooterLinks = [
    { title: dictionary.termsPrivacy.termsOfService, url: 'https://www.jetprotocol.io/legal/terms-of-service' },
    { title: dictionary.termsPrivacy.privacyPolicy, url: 'https://www.jetprotocol.io/legal/privacy-policy' },
    { title: dictionary.termsPrivacy.glossary, url: 'https://docs.jetprotocol.io/jet-protocol/terms-and-definitions' }
  ];

  return (
    <div className={`navbar-container flex-centered ${drawerOpened ? 'drawer-open' : ''}`}>
      {/* Desktop Nav */}
      <nav className="desktop flex align-center justify-between">
        <Link className="nav-logo flex-centered" to="/">
          <img className="logo" src="img/jet/jet_logo_white.png" width="100%" height="auto" alt="Jet Protocol" />
          <span className="green-text">V2 BETA</span>
        </Link>
        <div className="nav-links flex-centered">
          {navLinks.map(link => (
            <Link key={link.title} to={link.route} className={`nav-link ${pathname === link.route ? 'active' : ''}`}>
              {link.title}
            </Link>
          ))}
          <SettingFilled className="icon-btn" onClick={() => setOpen(true)} />
          <Button
            className="flex-centered"
            style={{ textTransform: 'unset' }}
            title={connected ? dictionary.settings.disconnect : dictionary.settings.connect}
            onClick={() => (connected ? disconnect() : setConnecting(true))}>
            <WalletIcon width="20px" />
            {connected
              ? `${shortenPubkey(publicKey ? publicKey.toString() : '')} ${dictionary.settings.connected.toUpperCase()}`
              : dictionary.settings.connect.toUpperCase()}
          </Button>
        </div>
      </nav>
      {/* Mobile Nav */}
      <nav className="mobile flex align-center justify-between">
        <SettingFilled className="icon-btn" onClick={() => setOpen(true)} />
        <Link className="nav-logo flex-centered" to="/">
          <img src="img/jet/jet_logo_white.png" width="100%" height="auto" alt="Jet Protocol" />
          <span className="green-text">V2 BETA</span>
        </Link>
        <div
          className={`hamburger flex align-center justify-between column ${drawerOpened ? 'close' : ''}`}
          onClick={() => setDrawerOpened(!drawerOpened)}>
          <span></span>
          <span></span>
          <span></span>
        </div>
        <div className="drawer flex align-center justify-between column">
          <div className="drawer-top flex-centered column">
            {navLinks.map(link => (
              <Link
                key={link.title}
                to={link.route}
                className={`nav-link ${pathname === link.route ? 'active' : ''}`}
                onClick={() => setDrawerOpened(false)}>
                {link.title}
              </Link>
            ))}
            <Button
              className="flex-centered small-btn"
              style={{ textTransform: 'unset' }}
              title={connected ? dictionary.settings.disconnect : dictionary.settings.connect}
              onClick={() => {
                if (connected) {
                  disconnect();
                } else {
                  setConnecting(true);
                  setDrawerOpened(false);
                }
              }}>
              <WalletIcon width="20px" />
              {connected
                ? shortenPubkey(publicKey ? publicKey.toString() : '')
                : dictionary.settings.connect.toUpperCase()}
            </Button>
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
