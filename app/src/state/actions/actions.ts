import { atom } from 'recoil';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { TokenAmount, PoolAction, Pool } from '@jet-lab/margin';

// Current action (controls action modals)
export const actionOptions: PoolAction[] = ['deposit', 'withdraw', 'borrow', 'repay', 'swap', 'transfer'];
export const CurrentAction = atom({
  key: 'currentAction',
  default: undefined as PoolAction | undefined
});

// Any time we need to refresh data, trigger this state change throughout relevant Wrappers
export const ACTION_REFRESH_INTERVAL = 4000;
export const ActionRefresh = atom({
  key: 'actionRefresh',
  default: false as boolean
});

// Token input for action modals
export const TokenInputAmount = atom({
  key: 'tokenInputAmount',
  default: TokenAmount.zero(Math.log10(LAMPORTS_PER_SOL)) as TokenAmount
});
export const TokenInputString = atom({
  key: 'tokenInputString',
  default: '0' as string
});

// Max trade amounts
export const MaxTradeAmounts = atom({
  key: 'maxTradeAmounts',
  default: {} as Record<PoolAction, TokenAmount>
});

// Current output token for swaps
export const CurrentSwapOutput = atom({
  key: 'currentSwapOutput',
  default: undefined as Pool | undefined,
  dangerouslyAllowMutability: true
});
