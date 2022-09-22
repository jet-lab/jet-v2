import { useRecoilState, useSetRecoilState, useRecoilValue, useResetRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { PoolAction } from '@jet-lab/margin';
import { Dictionary } from '../../../state/settings/localization/localization';
import { WalletTokens } from '../../../state/user/walletTokens';
import { WalletModal, NewAccountModal } from '../../../state/modals/modals';
import { Accounts } from '../../../state/user/accounts';
import { actionOptions, CurrentAction, TokenInputAmount, TokenInputString } from '../../../state/actions/actions';
import { Typography, Button } from 'antd';
import { ActionIcon } from '../ActionIcon';

// Footer of the Account Snapshot, where users can execute actions
export function SnapshotFooter(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const setWalletModalOpen = useSetRecoilState(WalletModal);
  const { connected } = useWallet();
  const walletTokens = useRecoilValue(WalletTokens);
  const accounts = useRecoilValue(Accounts);
  const setNewAccountModalOpen = useSetRecoilState(NewAccountModal);
  const [currentAction, setCurrentAction] = useRecoilState(CurrentAction);
  const resetTokenInputAmount = useResetRecoilState(TokenInputAmount);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const { Text } = Typography;

  // Either set current action or prompt wallet connection (if not connected)
  function setActionOrConnect(action?: PoolAction) {
    if (connected && walletTokens && !accounts.length) {
      setNewAccountModalOpen(true);
    } else if (connected && walletTokens) {
      resetTokenInputString();
      resetTokenInputAmount();
      setCurrentAction(action);
    } else {
      setWalletModalOpen(true);
    }
  }

  return (
    <div className="account-snapshot-footer flex-centered">
      {actionOptions.map(action => {
        // We won't use 'swap', because it has its own view
        if (action !== 'swap' && action !== 'repayFromDeposit') {
          return (
            <Button
              key={action}
              className={currentAction === action ? 'active' : ''}
              onClick={() => setActionOrConnect(action)}
              disabled={!walletTokens || !accounts.length}>
              <ActionIcon action={action} />
              <Text>{dictionary.actions[action].title}</Text>
            </Button>
          );
        }

        return undefined;
      })}
    </div>
  );
}
