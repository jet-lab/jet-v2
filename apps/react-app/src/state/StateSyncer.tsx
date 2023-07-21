import { useMainConfigSyncer } from './config/marginConfig';
import { usePoolsSyncer } from './pools/pools';
import { useWalletTokensSyncer } from './user/walletTokens';
import { useAccountsSyncer } from './user/accounts';
import { useFixedTermSync } from './fixed-term/fixed-term-market-sync';
// import { useProtocolClientSyncer } from './protocol/client';

export const StateSyncer = (): JSX.Element => {
  useMainConfigSyncer();
  usePoolsSyncer();
  useWalletTokensSyncer();
  useAccountsSyncer();
  useFixedTermSync();
  return <></>;
};

export default StateSyncer;
