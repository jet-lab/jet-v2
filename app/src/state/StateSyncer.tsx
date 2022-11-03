import { useMainConfigSyncer } from './config/marginConfig';
import { usePoolsSyncer } from './pools/pools';
import { useWalletTokensSyncer } from './user/walletTokens';
import { useAccountsSyncer } from './user/accounts';
import { useSplSwapSyncer } from './swap/splSwap';
import { useFixedTermSync } from './fixed-market/fixed-term-market-sync';

export function StateSyncer(): JSX.Element {
  useMainConfigSyncer();
  usePoolsSyncer();
  useWalletTokensSyncer();
  useAccountsSyncer();
  useSplSwapSyncer();
  useFixedTermSync();

  return <></>;
}

export default StateSyncer;
