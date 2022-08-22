import { useRecoilValue } from 'recoil';
import { WalkthroughModal } from '../../state/modals/modals';
import { WalletInit } from '../../state/user/walletTokens';
import { Accounts } from '../../state/user/accounts';
import { WalletMessage } from './WalletMessage';
import { AccountMessage } from './AccountMessage';

export function ConnectionFeedback(): JSX.Element {
  const walkthroughModalOpen = useRecoilValue(WalkthroughModal);
  const walletInit = useRecoilValue(WalletInit);
  const accounts = useRecoilValue(Accounts);

  if (!walletInit && !walkthroughModalOpen) {
    return <WalletMessage />;
  } else if (walletInit && !accounts.length && !walkthroughModalOpen) {
    return <AccountMessage />;
  } else {
    return <></>;
  }
}
