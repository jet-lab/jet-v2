import { FixedTermMarket, MarginAccount } from '@jet-lab/margin';
import { ReactNode, useEffect, useState } from 'react';

interface AutoRollChecksProps {
  market: FixedTermMarket;
  marginAccount?: MarginAccount;
  children: (args: { hasConfig: boolean }) => ReactNode;
}

export const AutoRollChecks = ({ children, market, marginAccount }: AutoRollChecksProps) => {
  const [hasConfig, setHasConfig] = useState(false);

  useEffect(() => {
    if (market && marginAccount) {
      console.log('Autoroll Config: checking market config');
      market.fetchMarginUser(marginAccount).then(response => {
        if (!response) {
          console.warn('Autoroll Config: No user found in this market');
          return;
        }

        if (!response.borrowRollConfig) {
          console.warn('Autoroll config: No borrow config found in this market');
          return;
        }

        if (!response.lendRollConfig) {
          console.warn('Autoroll config: No lend config found in this market.');
          return;
        }
        setHasConfig(true);
      });
    }
  }, [market, marginAccount]);

  return <>{children({ hasConfig })}</>
};
