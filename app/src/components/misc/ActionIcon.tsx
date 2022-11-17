import { useRecoilValue } from 'recoil';
import { CurrentAction } from '@state/actions/actions';
import { PoolAction } from '@jet-lab/margin';
import DepositIcon from '@assets/icons/function-deposit.svg';
import WithdrawIcon from '@assets/icons/function-withdraw.svg';
import BorrowIcon from '@assets/icons/function-borrow.svg';
import RepayIcon from '@assets/icons/function-repay.svg';
import SwapIcon from '@assets/icons/function-swap.svg';
import TransferIcon from '@assets/icons/function-transfer.svg';

// Return the correlated icon for a user action
export function ActionIcon(props: {
  action?: PoolAction;
  className?: string;
  style?: React.CSSProperties;
}): JSX.Element {
  const currentAction = useRecoilValue(CurrentAction);
  const action = props.action ?? currentAction;

  switch (action) {
    case 'deposit':
      return <DepositIcon className={`jet-icon ${props.className ?? ''}`} style={props.style} />;
    case 'borrow':
      return <BorrowIcon className={`jet-icon ${props.className ?? ''}`} style={props.style} />;
    case 'withdraw':
      return <WithdrawIcon className={`jet-icon ${props.className ?? ''}`} style={props.style} />;
    case 'repay':
      return <RepayIcon className={`jet-icon ${props.className ?? ''}`} style={props.style} />;
    case 'swap':
      return <SwapIcon className={`jet-icon ${props.className ?? ''}`} style={props.style} />;
    case 'transfer':
      return <TransferIcon className={`jet-icon ${props.className ?? ''}`} style={props.style} />;
    default:
      return <></>;
  }
}
