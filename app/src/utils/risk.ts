import { useRecoilValue } from 'recoil';
import { MarginAccount, Pool, PoolAction, TokenAmount } from '@jet-lab/margin';
import { CurrentPool } from '../state/borrow/pools';
import { CurrentAccount } from '../state/user/accounts';
import { CurrentAction, MaxTradeAmounts, TokenInputAmount } from '../state/actions/actions';

// Project user's risk from an action
export function useProjectedRisk(
  marginPool?: Pool,
  marginAccount?: MarginAccount,
  poolAction?: PoolAction | undefined,
  inputAmount?: TokenAmount
) {
  const currentPool = useRecoilValue(CurrentPool);
  const pool = marginPool ?? currentPool;
  const currentAccount = useRecoilValue(CurrentAccount);
  const account = marginAccount ?? currentAccount;
  const currentAction = useRecoilValue(CurrentAction);
  const action = poolAction ?? currentAction;
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const amount = inputAmount ?? tokenInputAmount;
  const maxTradeAmounts = useRecoilValue(MaxTradeAmounts);
  const max = action ? maxTradeAmounts[action] : undefined;

  const projectedRiskIndicator =
    pool && account && action && amount && !amount.isZero() && max && !amount.gt(max)
      ? pool.projectAfterAction(account, amount.tokens, action).riskIndicator
      : account?.riskIndicator ?? 0;

  return projectedRiskIndicator;
}

// Account risk based on required and effective collateral
export type RiskLevel = 'low' | 'moderate' | 'high';
export const riskLevels: RiskLevel[] = ['low', 'moderate', 'high'];
export function useRiskLevel(riskIndicator?: number): RiskLevel {
  const currentAccount = useRecoilValue(CurrentAccount);
  const risk = riskIndicator ?? currentAccount?.riskIndicator;

  if (risk && risk >= MarginAccount.RISK_CRITICAL_LEVEL) {
    return 'high';
  } else if (risk && risk >= MarginAccount.RISK_WARNING_LEVEL) {
    return 'moderate';
  } else {
    return 'low';
  }
}

// Return a usable class/type for styling based on risk
export function useRiskStyle(projectedRisk?: number) {
  const riskLevel = useRiskLevel(projectedRisk);

  if (riskLevel === 'high') {
    return 'danger';
  } else if (riskLevel === 'moderate') {
    return 'warning';
  } else {
    return 'success';
  }
}
