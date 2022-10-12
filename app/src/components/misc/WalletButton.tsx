import { useSetRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { Dictionary } from '../../state/settings/localization/localization';
import { WalletTokens } from '../../state/user/walletTokens';
import { Accounts } from '../../state/user/accounts';
import { WalletModal } from '../../state/modals/modals';
import { formatPubkey } from '../../utils/format';
import { notify } from '../../utils/notify';
import { Button } from 'antd';
import WalletIcon from '../../assets/icons/wallet-icon.svg';

// Button for connecting and disconnecting the user's wallet to the app
export function WalletButton(props: { mobile?: boolean }): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { wallet, connected, disconnect, publicKey } = useWallet();
  const resetWalletTokens = useResetRecoilState(WalletTokens);
  const resetAccounts = useResetRecoilState(Accounts);
  const setWalletModalOpen = useSetRecoilState(WalletModal);

  // If not connected open modal otherwise disconnect, reset and notify
  async function handleClick() {
    // If already connected, then we're disconnecting
    if (connected) {
      // Grab wallet properties for notification of disconnect
      const walletName = wallet?.adapter.name ?? '';
      const pubKey = publicKey;
      // Disconnect the wallet
      await disconnect();
      // Reset relevant state
      resetWalletTokens();
      resetAccounts();
      // Notify user of disconnection
      notify(
        dictionary.notifications.wallet.disconnect.message,
        dictionary.notifications.wallet.disconnect.description
          .replaceAll('{{WALLET_NAME}}', walletName)
          .replaceAll('{{PUBLIC_KEY}}', formatPubkey(pubKey?.toString() ?? '')),
        'warning'
      );
      // If not already connected, prompt user to connect wallet
    } else {
      setWalletModalOpen(true);
    }
  }

  // Returns the inner content of the wallet button
  function getWalletBtnContent() {
    let content = dictionary.settingsModal.wallet.connect.toUpperCase();
    // If connected, show public key (truncated) and (if not on mobile) "Connected" message
    if (connected && publicKey) {
      content = formatPubkey(publicKey.toString());
      if (!props.mobile) {
        content += ' ' + dictionary.settingsModal.wallet.connected.toUpperCase();
      }
    }

    return content;
  }

  // Returns the title of the wallet button
  function getWalletBtnTitle() {
    let title = dictionary.settingsModal.wallet.connect;
    // If connected, show title as "Disconnect" prompt
    if (connected) {
      title = dictionary.settingsModal.wallet.disconnect;
    }

    return title;
  }

  return (
    <Button className="wallet-button flex-centered" title={getWalletBtnTitle()} onClick={handleClick}>
      <WalletIcon width="25px" />
      {getWalletBtnContent()}
    </Button>
  );
}
