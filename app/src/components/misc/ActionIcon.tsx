import { useRecoilValue } from 'recoil';
import { PoolAction } from '@jet-lab/margin';
import { CurrentAction } from '../../state/actions/actions';
import { ReactComponent as DepositIcon } from '../../styles/icons/function-deposit.svg';
import { ReactComponent as WithdrawIcon } from '../../styles/icons/function-withdraw.svg';
import { ReactComponent as BorrowIcon } from '../../styles/icons/function-borrow.svg';
import { ReactComponent as RepayIcon } from '../../styles/icons/function-repay.svg';
import { ReactComponent as SwapIcon } from '../../styles/icons/function-swap.svg';
import { ReactComponent as TransferIcon } from '../../styles/icons/function-transfer.svg';

export function ActionIcon(props: { action?: PoolAction; style?: React.CSSProperties }): JSX.Element {
  const currentAction = useRecoilValue(CurrentAction);
  const a = props.action ?? currentAction;
  let icon: JSX.Element = <></>;
  if (a === 'deposit') {
    icon = <DepositIcon className="jet-icon" style={props.style} />;
  } else if (a === 'borrow') {
    icon = <BorrowIcon className="jet-icon" style={props.style} />;
  } else if (a === 'withdraw') {
    icon = <WithdrawIcon className="jet-icon" style={props.style} />;
  } else if (a === 'repay') {
    icon = <RepayIcon className="jet-icon" style={props.style} />;
  } else if (a === 'swap') {
    icon = <SwapIcon className="jet-icon" style={props.style} />;
  } else if (a === 'transfer') {
    icon = <TransferIcon className="jet-icon" style={props.style} />;
  }
  return icon;
}
