import { FixedTermMarket, MarginAccount } from '@jet-lab/margin';
import { ReactNode, useEffect, useState } from 'react';

interface AutoRollChecksProps {
  market: FixedTermMarket;
  marginAccount?: MarginAccount;
  children: (args: { hasConfig: boolean; refresh: () => Promise<void> }) => ReactNode;
}

export const AutoRollChecks = ({ children, market, marginAccount }: AutoRollChecksProps) => {
  const [hasConfig, setHasConfig] = useState(false);

  const refresh = async () => {
    if (market && marginAccount) {
      setHasConfig(false);
      console.log('Autoroll Config: checking market config');
      market.fetchMarginUser(marginAccount).then(async response => {
        console.log(response);
        if (!response) {
          console.warn('Autoroll Config: No user found in this market');
          return;
        }

        console.log(response.borrowRollConfig.limitPrice.toNumber());
        console.log(response.borrowRollConfig.rollTenor.toNumber());
        console.log(response.lendRollConfig.limitPrice.toNumber());

        if (response.borrowRollConfig.limitPrice.isZero()) {
          console.warn('Autoroll config: No borrow config found in this market');
          return;
        }

        if (response.lendRollConfig.limitPrice.isZero()) {
          console.warn('Autoroll config: No lend config found in this market.');
          return;
        }
        setHasConfig(true);
      });
    }
  };

  useEffect(() => {
    refresh();
  }, [market, marginAccount]);

  return <>{children({ hasConfig, refresh })}</>;
};
