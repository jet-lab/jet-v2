import { useRecoilValue, useSetRecoilState } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { WalletInit, WalletLoading } from '../../state/user/walletTokens';
import { WalletModal } from '../../state/modals/modals';
import { Button } from 'antd';
import { LoadingOutlined } from '@ant-design/icons';
import { ReactComponent as WalletLockedIcon } from '../../styles/icons/wallet-locked.svg';

// A feedback message overlaying components that can only used with a connected wallet
export function WalletMessage(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const walletLoading = useRecoilValue(WalletLoading);
  const walletInit = useRecoilValue(WalletInit);
  const setWalletModalOpen = useSetRecoilState(WalletModal);

  // Conditional render based on status of wallet connection
  function renderWalletMessage() {
    let render = (
      <div className="flex-centered column">
        <WalletLockedIcon className="jet-icon" />
        <Button className="function-btn" size="small" onClick={() => setWalletModalOpen(true)}>
          {dictionary.settingsModal.wallet.connectWallet}
        </Button>
      </div>
    );

    // If still loading initial wallet fetch, show loading spinner
    const initialWalletLoading = walletLoading && !walletInit;
    if (initialWalletLoading) {
      render = <LoadingOutlined />;
    }

    return render;
  }

  return <div className="overlay-message view-element-item">{renderWalletMessage()}</div>;
}
