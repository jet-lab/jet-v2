import { useRecoilState, useSetRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { PoolAction } from '@jet-lab/margin';
import { Dictionary } from '../../../state/settings/localization/localization';
import { WalletInit } from '../../../state/user/walletTokens';
import { WalletModal, NewAccountModal } from '../../../state/modals/modals';
import { Accounts, AccountsInit } from '../../../state/user/accounts';
import { actionOptions, CurrentAction } from '../../../state/actions/actions';
import { Typography, Button } from 'antd';
import { ActionIcon } from '../ActionIcon';

export function SnapshotFooter(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const setWalletModalOpen = useSetRecoilState(WalletModal);
  const { connected } = useWallet();
  const walletInit = useRecoilValue(WalletInit);
  const accounts = useRecoilValue(Accounts);
  const accountsInit = useRecoilValue(AccountsInit);
  const setNewAccountModalOpen = useSetRecoilState(NewAccountModal);
  const [currentAction, setCurrentAction] = useRecoilState(CurrentAction);
  const { Text } = Typography;

  // Either set current action or prompt wallet connection (if not connected)
  function setActionOrConnect(action?: PoolAction) {
    if (connected && walletInit && !accounts.length) {
      setNewAccountModalOpen(true);
    } else if (connected && walletInit) {
      setCurrentAction(action);
    } else {
      setWalletModalOpen(true);
    }
  }

  return (
    <div className="account-snapshot-footer view-element-item view-element-item-hidden flex-centered">
      {actionOptions.map(action => (
        <Button
          key={action}
          className={currentAction === action ? 'active' : ''}
          onClick={() => setActionOrConnect(action)}
          disabled={!walletInit || !accountsInit}>
          <ActionIcon action={action} />
          <Text>{dictionary.actions[action].title}</Text>
        </Button>
      ))}
    </div>
  );
}
