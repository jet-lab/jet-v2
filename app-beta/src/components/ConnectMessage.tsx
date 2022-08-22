import { useWallet } from '@solana/wallet-adapter-react';
import { Button } from 'antd';
import { useConnectWalletModal } from '../contexts/connectWalletModal';
import { useLanguage } from '../contexts/localization/localization';
import { LoadingOutlined } from '@ant-design/icons';

export function ConnectMessage(): JSX.Element {
  const { dictionary } = useLanguage();
  const { connected } = useWallet();
  const { setConnecting } = useConnectWalletModal();

  return (
    <div className="connect-message flex-centered" onClick={() => setConnecting(true)}>
      {!connected ? (
        <Button data-testid="connect-wallet-btn" type="dashed">
          {dictionary.settings.connect}&nbsp;{dictionary.settings.wallet}
        </Button>
      ) : (
        <LoadingOutlined className="green-text" style={{ fontSize: 25 }} />
      )}
    </div>
  );
}
