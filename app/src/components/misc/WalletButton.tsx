import { useSetRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { Dictionary } from '../../state/settings/localization/localization';
import { WalletTokens, WalletInit } from '../../state/user/walletTokens';
import { Accounts, AccountsInit } from '../../state/user/accounts';
import { WalletModal } from '../../state/modals/modals';
import { formatPubkey } from '../../utils/format';
import { notify } from '../../utils/notify';
import { Button } from 'antd';
import { ReactComponent as WalletIcon } from '../../styles/icons/wallet-icon.svg';

export function WalletButton(props: { mobile?: boolean }): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { wallet, connected, disconnect, publicKey } = useWallet();
  const resetWalletInit = useResetRecoilState(WalletInit);
  const resetWalletTokens = useResetRecoilState(WalletTokens);
  const resetAccountsInit = useResetRecoilState(AccountsInit);
  const resetAccounts = useResetRecoilState(Accounts);
  const setWalletModalOpen = useSetRecoilState(WalletModal);

  // If not connected open modal otherwise disconnect, reset and notify
  async function handleClick() {
    if (connected) {
      const walletName = wallet?.adapter.name ?? '';
      const pubKey = publicKey;
      await disconnect();
      resetWalletInit();
      resetWalletTokens();
      resetAccountsInit();
      resetAccounts();
      notify(
        dictionary.notifications.wallet.disconnect.message,
        dictionary.notifications.wallet.disconnect.description
          .replaceAll('{{WALLET_NAME}}', walletName)
          .replaceAll('{{PUBLIC_KEY}}', formatPubkey(pubKey?.toString() ?? '')),
        'warning'
      );
    } else {
      setWalletModalOpen(true);
    }
  }

  return (
    <Button
      className="wallet-button flex-centered"
      style={{ textTransform: 'unset' }}
      title={connected ? dictionary.settingsModal.wallet.disconnect : dictionary.settingsModal.wallet.connect}
      onClick={handleClick}>
      <WalletIcon width="25px" />
      {connected
        ? `${formatPubkey(publicKey ? publicKey.toString() : '')} ${
            props.mobile ? '' : dictionary.settingsModal.wallet.connected.toUpperCase()
          }`
        : dictionary.settingsModal.wallet.connect.toUpperCase()}
    </Button>
  );
}
