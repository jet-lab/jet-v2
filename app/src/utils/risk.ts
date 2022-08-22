import { useRecoilValue } from 'recoil';
import { MarginAccount } from '@jet-lab/margin';
import { CurrentAccount } from '../state/user/accounts';

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
