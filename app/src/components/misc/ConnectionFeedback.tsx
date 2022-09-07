import { useRecoilValue } from 'recoil';
import { WalkthroughModal } from '../../state/modals/modals';
import { WalletInit } from '../../state/user/walletTokens';
import { Accounts } from '../../state/user/accounts';
import { WalletMessage } from './WalletMessage';
import { AccountMessage } from './AccountMessage';

// Covers any component that would require a wallet connection / margin account to use
export function ConnectionFeedback(): JSX.Element {
  const walkthroughModalOpen = useRecoilValue(WalkthroughModal);
  const walletInit = useRecoilValue(WalletInit);
  const accounts = useRecoilValue(Accounts);
  const promptWalletConnection = !walletInit && !walkthroughModalOpen;
  const promptAccountCreation = walletInit && !accounts.length && !walkthroughModalOpen;

  if (promptWalletConnection) {
    return <WalletMessage />;
  } else if (promptAccountCreation) {
    return <AccountMessage />;
  } else {
    return <></>;
  }
}
