import { useEffect } from 'react';
import { useRecoilValue, useResetRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import LogRocket from 'logrocket';
import { Dictionary } from '../../state/settings/localization/localization';
import { LightTheme } from '../../state/settings/settings';
import { WalletModal as WalletModalState } from '../../state/modals/modals';
import { formatPubkey } from '../../utils/format';
import { notify } from '../../utils/notify';
import { Modal, Divider, Typography } from 'antd';
import { ReactComponent as ArrowIcon } from '../../styles/icons/arrow-icon.svg';

export function WalletModal(): JSX.Element {
  const { wallets, wallet, select, publicKey } = useWallet();
  const dictionary = useRecoilValue(Dictionary);
  const lightTheme = useRecoilValue(LightTheme);
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

      const logRocketProject = process.env.REACT_APP_LOGROCKET_PROJECT;
      if (logRocketProject) {
        LogRocket.init(logRocketProject);
        LogRocket.identify(publicKey.toBase58());
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [publicKey]);

  if (WalletModalOpen && !publicKey) {
    return (
      <Modal visible className="wallet-modal" footer={null} onCancel={resetWalletModal}>
        <div className="flex-centered column">
          <img src="img/jet/jet_logo.png" width="120px" height="auto" alt="Jet Protocol" />
          <Text>{dictionary.settingsModal.wallet.worldOfDefi}</Text>
          <Divider />
          <div className="wallets flex-centered column">
            {wallets.map(({ adapter }) => (
              <div
                key={adapter.name}
                className={`wallet flex align-center justify-between
                  ${wallet?.adapter.name === adapter.name ? 'active' : ''}`}
                onClick={() => {
                  select(adapter.name);
                }}>
                <div className="flex-centered">
                  <img
                    src={`img/wallets/${adapter.name.toLowerCase()}${
                      adapter.name === 'MathWallet' ? (lightTheme ? '_black' : '_white') : ''
                    }.png`}
                    width="30px"
                    height="auto"
                    alt={`${adapter.name} Logo`}
                  />
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
