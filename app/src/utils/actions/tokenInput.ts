import { useRecoilValue } from 'recoil';
import { feesBuffer, MarginAccount } from '@jet-lab/margin';
import { Dictionary } from '../../state/settings/localization/localization';
import { WalletTokens } from '../../state/user/walletTokens';
import { AccountNames, CurrentAccount } from '../../state/user/accounts';
import { Pools, CurrentPool } from '../../state/pools/pools';
import { CurrentAction, TokenInputAmount } from '../../state/actions/actions';
import { NewAccountModal } from '../../state/modals/modals';
import { formatRiskIndicator } from '../format';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';

// Check if user input should be disabled and return the relevant message
export function useTokenInputDisabledMessage(account?: MarginAccount): string {
  const dictionary = useRecoilValue(Dictionary);
  const walletTokens = useRecoilValue(WalletTokens);
  const accountNames = useRecoilValue(AccountNames);
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const currentAccount = useRecoilValue(CurrentAccount);
  const marginAccount = account ?? currentAccount;
  const currentAction = useRecoilValue(CurrentAction);
  const newAccountModalOpen = useRecoilValue(NewAccountModal);
  const poolPosition = marginAccount && currentPool?.symbol && marginAccount.poolPositions[currentPool.symbol];
  if (!pools || !currentPool || !marginAccount) {
    return '';
  }

  const tokenSymbol = currentPool.symbol;
  let disabledMessage = '';

  // Display message if user doesn't have enough SOL to cover fees
  if (walletTokens && walletTokens.map.SOL.amount.lamports.lte(feesBuffer)) {
    return (disabledMessage = ` ${dictionary.actions.deposit.disabledMessages.notEnoughSolForFees} 
      Please make sure you have at least ${ (feesBuffer.toNumber() / LAMPORTS_PER_SOL).toString() } SOL in your wallet.`)
  };

  if (currentAction === 'deposit') {
    // No wallet balance to deposit
    if (walletTokens && !walletTokens.map[tokenSymbol].amount.tokens) {
      disabledMessage = dictionary.actions.deposit.disabledMessages.noBalanceForDeposit.replaceAll(
        '{{ASSET}}',
        tokenSymbol
      );
    }
  } else if (currentAction === 'withdraw') {
    // No collateral to withdraw
    if (poolPosition && !poolPosition.depositBalance.tokens) {
      disabledMessage = dictionary.actions.withdraw.disabledMessages.noDepositsForWithdraw.replaceAll(
        '{{ASSET}}',
        tokenSymbol
      );
      // User is above max risk level
    } else if (
      marginAccount.summary.borrowedValue &&
      marginAccount.riskIndicator >= MarginAccount.RISK_LIQUIDATION_LEVEL
    ) {
      disabledMessage = dictionary.actions.disabledMessages.aboveMaxRisk;
      // No liquidity for withdraw
    } else if (!pools.tokenPools[tokenSymbol]?.vault.tokens) {
      disabledMessage = dictionary.actions.disabledMessages.notEnoughLiquidity;
    }
  } else if (currentAction === 'borrow') {
    // User has not deposited any collateral
    if (!marginAccount.summary.depositedValue) {
      disabledMessage = dictionary.actions.borrow.disabledMessages.noCollateralForBorrow;
      // User is above max risk level
    } else if (
      marginAccount.summary.borrowedValue &&
      marginAccount.riskIndicator >= MarginAccount.RISK_LIQUIDATION_LEVEL
    ) {
      disabledMessage = dictionary.actions.disabledMessages.aboveMaxRisk;
      // No liquidity in market to borrow from
    } else if (!pools.tokenPools[tokenSymbol]?.vault.tokens) {
      disabledMessage = dictionary.actions.disabledMessages.notEnoughLiquidity;
    }
  } else if (currentAction === 'repay' || currentAction === 'repayFromDeposit') {
    // User has no loan balance to repay
    if (poolPosition && !poolPosition.loanBalance.tokens) {
      disabledMessage = dictionary.actions.repay.disabledMessages.noDebtForRepay.replaceAll('{{ASSET}}', tokenSymbol);
    }
  } else if (currentAction === 'swap') {
    // No account balance to swap
    if (poolPosition && !poolPosition.depositBalance.tokens) {
      disabledMessage = dictionary.actions.swap.disabledMessages.noBalanceForSwap
        .replaceAll('{{ASSET}}', tokenSymbol)
        .replaceAll('{{WALLET_OR_ACCOUNT}}', dictionary.common.account.toLowerCase());
    }
  } else if (currentAction === 'transfer') {
    // User has no account balance to transfer
    if (poolPosition && !poolPosition.depositBalance.tokens) {
      let accountName = '';
      const accountMatch = Object.keys(accountNames).filter(key => key === marginAccount.address.toString())[0];
      if (accountMatch) {
        accountName = accountNames[accountMatch];
      }

      disabledMessage = dictionary.actions.transfer.disabledMessages.noBalanceForTransfer
        .replaceAll('{{ASSET}}', tokenSymbol)
        .replaceAll('{{ACCOUNT_NAME}}', accountName);
    }
  } else if (newAccountModalOpen) {
    // User is opening new account and has no SOL to cover the rent fee
    if (walletTokens && !walletTokens.map[tokenSymbol].amount.tokens) {
      disabledMessage = dictionary.actions.newAccount.disabledMessages.noSolForRentFee;
    }
  }

  return disabledMessage;
}

// Check if user input should be warned
export function useTokenInputWarningMessage(account?: MarginAccount | undefined): string {
  const dictionary = useRecoilValue(Dictionary);
  const currentPool = useRecoilValue(CurrentPool);
  const currentAccount = useRecoilValue(CurrentAccount);
  const marginAccount = account ?? currentAccount;
  const currentAction = useRecoilValue(CurrentAction);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const canProjectRisk =
    currentPool &&
    currentAccount &&
    currentAction &&
    !(currentAction === 'swap' || currentAction === 'transfer') &&
    !tokenInputAmount.isZero();
  const defaultRiskProjection = currentAccount?.riskIndicator ?? 0;
  const projectedRiskIndicator = canProjectRisk
    ? currentPool.projectAfterAction(currentAccount, tokenInputAmount.tokens, currentAction).riskIndicator
    : defaultRiskProjection;

  // If we should check for a risk level warning
  const checkForWarning =
    currentPool &&
    marginAccount &&
    !tokenInputAmount.isZero() &&
    (marginAccount?.summary.borrowedValue || currentAction === 'borrow');
  let warningMessage = '';
  if (checkForWarning) {
    // User's new Risk Level would be in liduidation territory (subject to market volatility)
    if (
      projectedRiskIndicator >= MarginAccount.RISK_WARNING_LEVEL &&
      projectedRiskIndicator <= MarginAccount.RISK_CRITICAL_LEVEL
    ) {
      warningMessage = dictionary.actions.warningMessages.nearMaxRisk
        .replaceAll('{{NEW_RISK}}', formatRiskIndicator(projectedRiskIndicator))
        .replaceAll('{{MAX_RISK}}', formatRiskIndicator(MarginAccount.RISK_LIQUIDATION_LEVEL));
    }
  }
  return warningMessage;
}

// Check if user input is dangerous / would cause error
export function useTokenInputErrorMessage(account?: MarginAccount | undefined, projectedRisk?: number): string {
  const dictionary = useRecoilValue(Dictionary);
  const currentPool = useRecoilValue(CurrentPool);
  const currentAction = useRecoilValue(CurrentAction);
  const walletTokens = useRecoilValue(WalletTokens);
  const currentAccount = useRecoilValue(CurrentAccount);
  const marginAccount = account ?? currentAccount;
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const projectedRiskIndicator =
    currentPool &&
    currentAccount &&
    currentAction &&
    !(currentAction === 'swap' || currentAction === 'transfer') &&
    !tokenInputAmount.isZero()
      ? currentPool.projectAfterAction(currentAccount, tokenInputAmount.tokens, currentAction).riskIndicator
      : currentAccount?.riskIndicator ?? 0;
  const risk = projectedRisk ?? projectedRiskIndicator;

  let errorMessage = '';
  if (marginAccount && !tokenInputAmount.isZero() && walletTokens) {
    // User's new Risk Level would be above our maximum
    if (risk >= MarginAccount.RISK_LIQUIDATION_LEVEL) {
      errorMessage = dictionary.actions.errorMessages.maxRiskLevel.replaceAll(
        '{{NEW_RISK}}',
        formatRiskIndicator(risk)
      );
    }
  }
  return errorMessage;
}
