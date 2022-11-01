import { useEffect } from 'react';
import { useRecoilValue, useResetRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import LogRocket from 'logrocket';
import { Dictionary } from '../../state/settings/localization/localization';
import { WalletModal as WalletModalState } from '../../state/modals/modals';
import { formatPubkey } from '../../utils/format';
import { notify } from '../../utils/notify';
import { Modal, Divider, Typography } from 'antd';
import ArrowIcon from '../../assets/icons/arrow-icon.svg';

// Modal to connect user's Solana wallet to app
export function WalletModal(): JSX.Element {
  const { wallets, wallet, select, publicKey } = useWallet();
  const dictionary = useRecoilValue(Dictionary);
  const WalletModalOpen = useRecoilValue(WalletModalState);
  const resetWalletModal = useResetRecoilState(WalletModalState);
  const { Text } = Typography;

  // Nofity on connection
  useEffect(() => {
    if (publicKey) {
      resetWalletModal();
      notify(
        dictionary.notifications.wallet.connect.message,
        dictionary.notifications.wallet.connect.description
          .replaceAll('{{WALLET_NAME}}', wallet?.adapter.name ?? '')
          .replaceAll('{{PUBLIC_KEY}}', formatPubkey(publicKey?.toString() ?? '')),
        'success'
      );

      // Initiate logRocket
      const logRocketProject = process.env.REACT_APP_LOGROCKET_PROJECT;
      if (logRocketProject) {
        LogRocket.init(logRocketProject);
        LogRocket.identify(publicKey.toBase58());
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [publicKey]);

  // Returns className for wallet and checks if 'active'
  function getWalletClassNames(walletName: string) {
    let className = 'wallet flex align-center justify-between';
    if (wallet && wallet.adapter.name === walletName) {
      className += ' active';
    }

    return className;
  }

  // Returns the img path for the wallet logo
  function getWalletLogoPath(walletName: string) {
    let path = `img/wallets/${walletName.toLowerCase()}`;
    // If Math Wallet, specify white or black logo (depending on theme)
    if (walletName === 'MathWallet') {
      path += '_white';
    }

    return path + '.png';
  }

  // If wallet modal is open and we're not already connected
  if (WalletModalOpen && !publicKey) {
    return (
      <Modal open className="wallet-modal" maskClosable={false} footer={null} onCancel={resetWalletModal}>
        <div className="flex-centered column">
          <img src="img/jet/jet_logo.png" width="120px" height="auto" alt="Jet Protocol" />
          <Text>{dictionary.settingsModal.wallet.worldOfDefi}</Text>
          <Divider />
          <div className="wallets flex-centered column">
            {wallets.map(({ adapter }) => (
              <div
                key={adapter.name}
                className={getWalletClassNames(adapter.name)}
                onClick={() => select(adapter.name)}>
                <div className="flex-centered">
                  <img src={getWalletLogoPath(adapter.name)} width="30px" height="auto" alt={`${adapter.name} Logo`} />
                  <p className="center-text">{adapter.name}</p>
                </div>
                <ArrowIcon width="25px" />
              </div>
            ))}
          </div>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
