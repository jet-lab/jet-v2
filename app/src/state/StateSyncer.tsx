import { useMarginConfigSyncer } from './config/marginConfig';
import { usePoolsSyncer } from './pools/pools';
import { useWalletTokensSyncer } from './user/walletTokens';
import { useAccountsSyncer } from './user/accounts';
import { useSplSwapSyncer } from './swap/splSwap';

// React component to utilize all sync hooks from bigger state
export function StateSyncer(): JSX.Element {
  useMarginConfigSyncer();
  usePoolsSyncer();
  useWalletTokensSyncer();
  useAccountsSyncer();
  useSplSwapSyncer();

  return <></>;
}
