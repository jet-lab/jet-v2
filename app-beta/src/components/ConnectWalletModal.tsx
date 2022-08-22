import { useEffect } from 'react';
import LogRocket from 'logrocket';
import { useWallet } from '@solana/wallet-adapter-react';
import { useConnectWalletModal } from '../contexts/connectWalletModal';
import { useLanguage } from '../contexts/localization/localization';
import { Modal, Divider } from 'antd';
import { ReactComponent as ArrowIcon } from '../styles/icons/arrow_icon.svg';

export function ConnectWalletModal(): JSX.Element {
  const { dictionary } = useLanguage();
  const { wallets, select, connected, wallet, publicKey } = useWallet();
  const { connecting, setConnecting } = useConnectWalletModal();

  useEffect(() => {
    if (connected) {
      setConnecting(false);

      const project = process.env.REACT_APP_LOGROCKET_PROJECT;
      if (project && publicKey) {
        LogRocket.init(project);
        LogRocket.identify(publicKey.toBase58());
      }
    }
  }, [connected, setConnecting, publicKey]);

  return (
    <Modal
      footer={null}
      visible={connecting && !connected}
      className="connect-modal"
      onCancel={() => setConnecting(false)}>
      <div className="flex-centered column">
        <div className="flex-centered">
          <img
            style={{ opacity: 0.8 }}
            className="logo"
            src="img/jet/jet_logo_white.png"
            width="120px"
            height="auto"
            alt="Jet Protocol"
          />
          <span className="green-text" style={{ fontWeight: 'bold', marginLeft: 4 }}>
            V2 BETA
          </span>
        </div>
        <span>{dictionary.settings.worldOfDefi}</span>
        <Divider />
        <div className="wallets flex-centered column">
          {wallets.map(w => (
            <div
              data-testid={`connect-wallet-${w.adapter.name}`}
              key={w.adapter.name}
              className={`wallet flex align-center justify-between
                ${wallet?.adapter.name === w.adapter.name ? 'active' : ''}`}
              onClick={() => {
                select(w.adapter.name);
              }}>
              <div className="flex-centered">
                <img
                  src={`img/wallets/${w.adapter.name.toLowerCase()}.png`}
                  width="30px"
                  height="auto"
                  alt={`${w.adapter.name} Logo`}
                />
                <p className="center-text">{w.adapter.name}</p>
              </div>
              <ArrowIcon width="25px" />
            </div>
          ))}
        </div>
      </div>
    </Modal>
  );
}
