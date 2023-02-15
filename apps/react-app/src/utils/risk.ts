import { useRecoilValue } from 'recoil';
import { MarginAccount, Pool, TokenAmount, PoolAction } from '@jet-lab/margin';
import { CurrentAccount } from '@state/user/accounts';
import { CurrentAction, MaxTradeAmounts, TokenInputAmount } from '@state/actions/actions';
import { useJetStore } from '@jet-lab/store';
import { Pools } from '@state/pools/pools';
import { useMemo } from 'react';

// Project user's risk from an action
export function useProjectedRisk(
  marginPool?: Pool,
  marginAccount?: MarginAccount,
  poolAction?: PoolAction,
  inputAmount?: TokenAmount,
  // If user is swapping
  minAmountOut?: TokenAmount,
  outputToken?: Pool,
  swapRepayWithProceeds?: boolean
) {
  const selectedPoolKey = useJetStore(state => state.selectedPoolKey);
  const pools = useRecoilValue(Pools);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );
  const pool = marginPool ?? currentPool;
  const currentAccount = useRecoilValue(CurrentAccount);
  const account = marginAccount ?? currentAccount;
  const currentAction = useRecoilValue(CurrentAction);
  const action = poolAction ?? currentAction;
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const amount = inputAmount ?? tokenInputAmount;
  const maxTradeAmounts = useRecoilValue(MaxTradeAmounts);
  const max = action && action !== 'transfer' ? maxTradeAmounts[action] : undefined;

  const canProjectAfterAction =
    pool && account && action && action !== 'transfer' && amount && !amount.isZero() && max && !amount.gt(max);
  const defaultActionProjection = account?.riskIndicator ?? 0;

  if (max && amount.gt(max)) {
    return Infinity;
  }
  const projectedRiskIndicator = canProjectAfterAction
    ? pool.projectAfterAction(
        account,
        amount.tokens,
        action,
        minAmountOut && minAmountOut.tokens,
        outputToken,
        swapRepayWithProceeds
      ).riskIndicator
    : defaultActionProjection;

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
