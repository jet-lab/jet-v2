import { useRecoilValue } from 'recoil';
import { WalletTokens } from '../../../state/user/walletTokens';
import { Accounts } from '../../../state/user/accounts';
import { WalletMessage } from './WalletMessage';
import { AccountMessage } from './AccountMessage';

// Covers any component that would require a wallet connection / margin account to use
export function ConnectionFeedback(): JSX.Element {
  const walletTokens = useRecoilValue(WalletTokens);
  const accounts = useRecoilValue(Accounts);
  const promptWalletConnection = !walletTokens;
  const promptAccountCreation = walletTokens && !accounts.length;

  if (promptWalletConnection) {
    return <WalletMessage />;
  } else if (promptAccountCreation) {
    return <AccountMessage />;
  } else {
    return <></>;
  }
}
