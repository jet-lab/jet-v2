import { FixedTermMarket, MarginAccount, price_to_rate } from '@jet-lab/margin';
import { ReactNode, useEffect, useState } from 'react';

interface AutoRollChecksProps {
  market: FixedTermMarket;
  marginAccount?: MarginAccount;
  children: (args: {
    borrowRate?: number;
    lendRate?: number;
    hasConfig: boolean;
    refresh: () => Promise<void>;
  }) => ReactNode;
}

export const AutoRollChecks = ({ children, market, marginAccount }: AutoRollChecksProps) => {
  const [hasConfig, setHasConfig] = useState(false);
  const [borrowRate, setBorrowRate] = useState<number>();
  const [lendRate, setLendRate] = useState<number>();

  const refresh = async () => {
    if (market && marginAccount) {
      setHasConfig(false);
      setBorrowRate(undefined);
      setLendRate(undefined);
      console.log('Autoroll Config: checking market config');
      market.fetchMarginUser(marginAccount).then(async response => {
        if (!response) {
          console.warn('Autoroll Config: No user found in this market');
          return;
        }

        if (response.borrowRollConfig.limit_price === BigInt(0)) {
          console.warn('Autoroll config: No borrow config found in this market');
          return;
        } else {
          setBorrowRate(
            Number(price_to_rate(response.borrowRollConfig.limit_price, response.borrowRollConfig.roll_tenor))
          );
        }

        if (response.lendRollConfig.limit_price === BigInt(0)) {
          console.warn('Autoroll config: No lend config found in this market.');
          return;
        } else {
          setLendRate(Number(price_to_rate(response.lendRollConfig.limit_price, response.borrowRollConfig.roll_tenor)));
        }
        setHasConfig(true);
      });
    }
  };

  useEffect(() => {
    refresh();
  }, [market, marginAccount]);

  return <>{children({ hasConfig, refresh, borrowRate, lendRate })}</>;
};
