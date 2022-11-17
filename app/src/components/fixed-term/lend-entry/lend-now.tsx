import { Button, InputNumber, Switch, Tooltip } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import { lendNow } from '@jet-lab/jet-bonds-client';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import BN from 'bn.js';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { CurrentAccount } from '@state/user/accounts';
import { useWallet } from '@solana/wallet-adapter-react';
import { useProvider } from '@utils/jet/provider';
import { CurrentPool, Pools } from '@state/pools/pools';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { useRecoilValue } from 'recoil';
import { useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { AllFixedMarketsAtom, MarketAndconfig } from '@state/fixed-market/fixed-term-market-sync';
import debounce from 'lodash.debounce';

interface RequestLoanProps {
  decimals: number;
  token: MarginTokenConfig;
  marketAndConfig: MarketAndconfig;
  marginConfig: MarginConfig;
}

export const LendNow = ({ token, decimals, marketAndConfig, marginConfig }: RequestLoanProps) => {
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const cluster = useRecoilValue(Cluster);
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const wallet = useWallet();
  const blockExplorer = useRecoilValue(BlockExplorer);
  const [amount, setAmount] = useState(new BN(0));
  const markets = useRecoilValue(AllFixedMarketsAtom);

  const marketLendOrder = async () => {
    let signature: string;
    try {
      signature = await lendNow({
        market: marketAndConfig.market,
        marginAccount,
        marginConfig,
        provider,
        walletAddress: wallet.publicKey,
        pools: pools.tokenPools,
        currentPool,
        amount,
        markets: markets.map(m => m.market)
      });
      notify(
        'Lend Successful',
        `Your lend order for ${amount.div(new BN(10 ** decimals))} ${token.name} was filled successfully`,
        'success',
        getExplorerUrl(signature, cluster, blockExplorer)
      );
    } catch (e) {
      console.log(e);
      notify(
        'Lend Order Failed',
        `Your lend order for ${amount.div(new BN(10 ** decimals))} ${token.name} failed`,
        'error',
        getExplorerUrl(e.signature, cluster, blockExplorer)
      );
    }
  };

  return (
    <div className="fixed-term order-entry-body">
      <div className="lend-now fixed-order-entry-fields">
        <label>
          Loan amount
          <InputNumber
            className="input-amount"
            onChange={debounce(e => setAmount(new BN(e * 10 ** decimals)), 300)}
            placeholder={'10,000'}
            min={0}
            formatter={value => `${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',')}
            controls={false}
            addonAfter={marketAndConfig.config.symbol}
          />
        </label>
      </div>

      <div className="auto-roll-controls">
        <Tooltip title="Coming soon...">
          <Switch disabled={true} />
        </Tooltip>
        Auto-roll Off
      </div>

      <div className="stats">
        <div className="stat-line">
          <span>Repayment Date</span>
          <span>
            {`in ${formatDuration(
              intervalToDuration({
                start: new Date(0),
                end: new Date(marketAndConfig.config.borrowDuration * 1000)
              })
            )}`}
          </span>
        </div>
        <div className="stat-line">
          <span>Repayment Amount</span>
          <span>??</span>
        </div>
        <div className="stat-line">
          <span>Total Interest</span>
          <span>??</span>
        </div>
        <div className="stat-line">
          <span>Interest Rate</span>
          <span>??</span>
        </div>
        <div className="stat-line">Risk Level</div>
        <div className="stat-line">
          <span>Auto Roll</span>
          <span>Off</span>
        </div>
      </div>
      <Button
        className="submit-button"
        disabled={!marketAndConfig?.market || amount.lte(new BN(0))}
        onClick={marketLendOrder}>
        Lend {marketToString(marketAndConfig.config)}
      </Button>
    </div>
  );
};
