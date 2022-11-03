import { Button, InputNumber, Switch, Tooltip } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import { createFixedLendOrder } from '@jet-lab/jet-bonds-client';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import BN from 'bn.js'
import { marketToString } from '@utils/jet/fixed-term-utils';
import { CurrentAccount } from '@state/user/accounts';
import { useWallet } from '@solana/wallet-adapter-react';
import { useProvider } from '@utils/jet/provider';
import { CurrentPool, Pools } from '@state/pools/pools';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { useRecoilValue } from 'recoil';
import { useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { MarketAndconfig } from '@state/fixed/fixed-term-market-sync';
import { formatWithCommas } from '@utils/format';

interface RequestLoanProps {
  decimals: number
  token: MarginTokenConfig
  marketAndConfig: MarketAndconfig
  marginConfig: MarginConfig
}

export const OfferLoan = ({
  token,
  decimals,
  marketAndConfig,
  marginConfig
}: RequestLoanProps) => {

  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const cluster = useRecoilValue(Cluster);
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const wallet = useWallet();
  const blockExplorer = useRecoilValue(BlockExplorer);
  const [amount, setAmount] = useState(new BN(0));
  const [basisPoints, setBasisPoints] = useState(new BN(0));

  const createLendOrder = async () => {
    let signature: string;
    try {
      signature = await createFixedLendOrder({
        market: marketAndConfig.market,
        marginAccount,
        marginConfig,
        provider,
        walletAddress: wallet.publicKey,
        pools: pools.tokenPools,
        currentPool,
        amount,
        basisPoints
      });
      notify(
        'Lend Order Created',
        `Your lend order for ${amount.div(new BN(10 ** decimals))} ${token.name} was created successfully`,
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

  return <div className="order-entry-body">
    <div className="fixed-order-entry-fields">
      <label>
        Loan amount
        <InputNumber
          onChange={e => setAmount(new BN(e * 10 ** decimals))}
          placeholder={'10,000'}
          min={0}
          formatter={formatWithCommas}
          controls={false}
          addonAfter={marketAndConfig.config.symbol}
        />
      </label>
      <label>
        Interest Rate
        <InputNumber
          onChange={e => {
            setBasisPoints(new BN(e * 100));
          }}
          placeholder={'1.5'}
          type="number"
          step={0.01}
          min={0}
          controls={false}
          addonAfter="%"
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
        <span>{`${formatDuration(
          intervalToDuration({
            start: new Date(0),
            end: new Date(marketAndConfig.config.borrowDuration * 1000)
          })
        )} from fill`}
          from fill
        </span>
      </div>
      <div className="stat-line">
        <span>Repayment Amount</span>
        <span>
          {formatWithCommas(((amount.toNumber() / 10 ** decimals) * (1 + basisPoints.toNumber() / 10000)).toFixed(token.precision))} {token.symbol}
        </span>
      </div>
      <div className="stat-line">
        <span>Total Interest</span>
        <span>
          {(amount.toNumber() / 10 ** decimals) * (basisPoints.toNumber() / 10000)} {token.symbol}
        </span>
      </div>
      <div className="stat-line">
        <span>Interest Rate</span>
        <span>{basisPoints.toNumber() / 100}%</span>
      </div>
      <div className="stat-line">Risk Level</div>
      <div className="stat-line">
        <span>Auto Roll</span>
        <span>Off</span>
      </div>
    </div>
    <Button disabled={!marketAndConfig?.market || basisPoints.lte(new BN(0)) || amount.lte(new BN(0))} onClick={createLendOrder}>
      Request {marketToString(marketAndConfig.config)} loan
    </Button>
  </div>
}