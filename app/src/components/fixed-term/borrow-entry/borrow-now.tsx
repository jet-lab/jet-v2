import { Button, InputNumber, Switch, Tooltip } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import { borrowNow, estimate_order_outcome, Order } from '@jet-lab/jet-bonds-client';
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
import { useCallback, useEffect, useMemo, useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { AllFixedMarketsOrderBooksAtom, MarketAndconfig, FixedMarketAtom } from '@state/fixed-market/fixed-term-market-sync';

interface RequestLoanProps {
  decimals: number;
  token: MarginTokenConfig;
  marketAndConfig: MarketAndconfig;
  marginConfig: MarginConfig;
}

export const BorrowNow = ({ token, decimals, marketAndConfig, marginConfig }: RequestLoanProps) => {
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const cluster = useRecoilValue(Cluster);
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const wallet = useWallet();
  const blockExplorer = useRecoilValue(BlockExplorer);
  const books = useRecoilValue(AllFixedMarketsOrderBooksAtom);
  const [amount, setAmount] = useState(new BN(0));

  const estimateOrderOutcome = useCallback((amount: BN) => {
    const bids = books[0].bids.sort((x, y) => Number(y.limit_price) - Number(x.limit_price))
    const result = estimate_order_outcome(
      BigInt(amount.toNumber()),
      marginAccount.address.toBuffer(),
      3,
      null,
      bids
    )
    console.log({
      vwap: Number(result.vwap),
      filled_base: Number(result.filled_base),
      filled_quote: Number(result.filled_quote),
      matches: Number(result.matches),
      unfilled_quote: Number(result.unfilled_quote)
    })
  }, [])

  useEffect(() => {
    console.log(books[0].bids)
    if (amount.gt(new BN(0)) && books[0].bids.length > 0 && marginAccount.address) {
      estimateOrderOutcome(amount)
    }
  }, [amount])

  const createBorrowOrder = async () => {
    let signature: string;
    try {
      signature = await borrowNow({
        market: marketAndConfig.market,
        marginAccount,
        marginConfig,
        provider,
        walletAddress: wallet.publicKey,
        pools: pools.tokenPools,
        currentPool,
        amount
      });
      notify(
        'Borrow Successful',
        `Your borrow order for ${amount.div(new BN(10 ** decimals))} ${token.name} was filled successfully`,
        'success',
        getExplorerUrl(signature, cluster, blockExplorer)
      );
    } catch (e) {
      console.log(e);
      notify(
        'Borrow Order Failed',
        `Your borrow order for ${amount.div(new BN(10 ** decimals))} ${token.name} failed`,
        'error',
        getExplorerUrl(e.signature, cluster, blockExplorer)
      );
    }
  };

  return (
    <div className="fixed-term order-entry-body">
      <div className="borrow-now fixed-order-entry-fields">
        <label>
          Loan amount
          <InputNumber
            className="input-amount"
            onChange={e => setAmount(new BN(e * 10 ** decimals))}
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
        onClick={createBorrowOrder}>
        Borrow {marketToString(marketAndConfig.config)}
      </Button>
    </div>
  );
};
