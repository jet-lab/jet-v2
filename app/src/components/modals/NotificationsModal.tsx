//import { useState } from 'react';
import { useRecoilValue, useResetRecoilState } from 'recoil';
//import { useWallet } from '@solana/wallet-adapter-react';
//import { Cluster, PreferredRpcNode, rpcNodes } from '../../state/settings/settings';
import { Dictionary } from '../../state/settings/localization/localization';
import { NotificationsModal as NotificationModalState } from '../../state/modals/modals';
//import { ApiProvider, DialectProvider } from '@dialectlabs/react';
// import {
//   DialectUiManagementProvider,
//   NotificationsModal as DialectModal,
//   Notifications as DialectNotifications,
//   ThemeProvider
// } from '@dialectlabs/react-ui';
import { Modal, Typography } from 'antd';
//import { PublicKey } from '@solana/web3.js';

//const JET_DIALECT_PUBKEY = new PublicKey('AainXWecQt5TjGQgw5R6oLNu9zvvQcS1kkVbG9UQqaP8');
export function NotificationsModal() {
  // const cluster = useRecoilValue(Cluster);
  // const preferredNode = useRecoilValue(PreferredRpcNode);
  const dictionary = useRecoilValue(Dictionary);
  //const wallet = useWallet();
  const notificationsModalOpen = useRecoilValue(NotificationModalState);
  const resetNotificationsModalOpen = useResetRecoilState(NotificationModalState);
  //const [showNotis, setShowNotis] = useState(false);
  const { Title } = Typography;

  if (notificationsModalOpen) {
    return (
      <Modal visible className="header-modal notifications-modal" onCancel={resetNotificationsModalOpen}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">{dictionary.notificationsModal.title}</Title>
        </div>
        {/*
        <DialectUiManagementProvider>
          <ThemeProvider>
            <ApiProvider>
              <DialectProvider>
                {showNotis ? (
                  <DialectNotifications
                    onModalClose={() => null}
                    onBackClick={() => null}
                    channels={['web3', 'email', 'telegram']}
                    notifications={[
                      {
                        name: dictionary.notificationsModal.riskLevelName,
                        detail: dictionary.notificationsModal.riskLevelDescription
                      }
                    ]}
                  />
                ) : (
                  <DialectModal
                    network={cluster}
                    publicKey={JET_DIALECT_PUBKEY}
                    dialectId={JET_DIALECT_PUBKEY.toString()}
                    wallet={wallet}
                    rpcUrl={
                      cluster === 'mainnet-beta' ? rpcNodes[preferredNode].mainnetBeta : rpcNodes[preferredNode].devnet
                    }
                    channels={['web3', 'email', 'telegram']}
                    notifications={[
                      {
                        name: dictionary.notificationsModal.riskLevelName,
                        detail: dictionary.notificationsModal.riskLevelDescription
                      }
                    ]}
                  />
                )}
              </DialectProvider>
            </ApiProvider>
          </ThemeProvider>
        </DialectUiManagementProvider>
                  */}
      </Modal>
    );
  } else {
    return <></>;
  }
}
