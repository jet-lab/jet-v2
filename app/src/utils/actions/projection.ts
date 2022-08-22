import { MarginAccount, Pool, TokenAmount } from '@jet-lab/margin';

// Get risk indicator
export const getProjectionAfterAction = (
  currentPool: Pool,
  currentAction: string,
  fromAccount: MarginAccount,
  tokenInputAmount: TokenAmount
): number | undefined => {
  switch (currentAction) {
    case 'deposit':
      return currentPool?.projectAfterDeposit(fromAccount, tokenInputAmount.tokens).riskIndicator;
    case 'withdraw':
      return currentPool?.projectAfterWithdraw(fromAccount, tokenInputAmount.tokens).riskIndicator;
    case 'borrow':
      return currentPool?.projectAfterDeposit(fromAccount, tokenInputAmount.tokens).riskIndicator;
    case 'repay':
      return currentPool?.projectAfterRepay(fromAccount, tokenInputAmount.tokens).riskIndicator;
    case 'swap':
      // TODO: Add correct projection after action from v2 lib
      // currentPool?.projectAfterDeposit(fromAccount, tokenInputAmount.tokens).riskIndicator
      break;
    case 'transfer':
      // TODO: Add correct projection after action from v2 lib
      // currentPool?.projectAfter(fromAccount, tokenInputAmount.tokens).riskIndicator
      break;
  }
};
