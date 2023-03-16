import { Button, InputNumber, Switch, Tooltip } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import {
  bigIntToBn,
  bnToBigInt,
  borrowNow,
  FixedTermProductModel,
  MarketAndConfig,
  OrderbookModel,
  TokenAmount
} from '@jet-lab/margin';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import BN from 'bn.js';
import { feesCalc, marketToString } from '@utils/jet/fixed-term-utils';
import { CurrentAccount } from '@state/user/accounts';
import { useWallet } from '@solana/wallet-adapter-react';
import { useProvider } from '@utils/jet/provider';
import { Pools } from '@state/pools/pools';
import { useRecoilRefresher_UNSTABLE, useRecoilValue } from 'recoil';
import { useEffect, useMemo, useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { AllFixedTermMarketsAtom, AllFixedTermMarketsOrderBooksAtom } from '@state/fixed-term/fixed-term-market-sync';
import debounce from 'lodash.debounce';
import { RateDisplay } from '../shared/rate-display';
import { useJetStore } from '@jet-lab/store';
import { LoadingOutlined } from '@ant-design/icons';

interface RequestLoanProps {
  decimals: number;
  token: MarginTokenConfig;
  marketAndConfig: MarketAndConfig;
  marginConfig: MarginConfig;
}

interface Forecast {
  repayAmount: number;
  interest: number;
  effectiveRate: number;
  selfMatch: boolean;
  fulfilled: boolean;
  riskIndicator?: number;
  unfilledQty: number;
  hasEnoughCollateral: boolean;
  fees: number;
}

export const BorrowNow = ({ token, decimals, marketAndConfig }: RequestLoanProps) => {
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const { selectedPoolKey } = useJetStore(state => ({
    selectedPoolKey: state.selectedPoolKey
  }));
  const pools = useRecoilValue(Pools);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );
  const wallet = useWallet();
  const [amount, setAmount] = useState<BN | undefined>();
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const refreshOrderBooks = useRecoilRefresher_UNSTABLE(AllFixedTermMarketsOrderBooksAtom);
  const [forecast, setForecast] = useState<Forecast>();

  const { cluster, explorer } = useJetStore(state => state.settings);

  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (amount) handleForecast(amount);
  }, [amount, marginAccount?.address, marketAndConfig]);

  const disabled =
    !marginAccount ||
    !wallet.publicKey ||
    !currentPool ||
    !pools ||
    amount?.lte(new BN(0)) ||
    !forecast?.effectiveRate ||
    forecast.selfMatch ||
    !forecast.fulfilled ||
    forecast.unfilledQty > 0 ||
    !forecast?.hasEnoughCollateral;

  const handleForecast = (amount: BN) => {
    if (bnToBigInt(amount) === BigInt(0)) {
      setForecast(undefined);
      return;
    }
    const orderbookModel = marketAndConfig.market.orderbookModel as OrderbookModel;
    try {
      const sim = orderbookModel.simulateTaker(
        'borrow',
        bnToBigInt(amount),
        undefined,
        marginAccount?.address.toBytes()
      );

      let correspondingPool = pools?.tokenPools[marketAndConfig.token.symbol];
      if (correspondingPool == undefined) {
        console.log('ERROR `correspondingPool` must be defined.');
        return;
      }

      const productModel = marginAccount
        ? FixedTermProductModel.fromMarginAccountPool(marginAccount, correspondingPool)
        : undefined;
      const setupCheckEstimate = productModel?.takerAccountForecast('borrow', sim, 'setup');
      const valuationEstimate = productModel?.takerAccountForecast('borrow', sim);

      const repayAmount = new TokenAmount(bigIntToBn(sim.filled_base_qty), token.decimals);
      const borrowedAmount = new TokenAmount(bigIntToBn(sim.filled_quote_qty), token.decimals);
      const unfilledQty = new TokenAmount(bigIntToBn(sim.unfilled_quote_qty - sim.matches), token.decimals);
      const totalInterest = repayAmount.sub(borrowedAmount);

      setForecast({
        repayAmount: repayAmount.tokens,
        interest: totalInterest.tokens,
        effectiveRate: sim.filled_vwar,
        selfMatch: sim.self_match,
        fulfilled: sim.filled_quote_qty >= sim.order_quote_qty - BigInt(1) * sim.matches, // allow 1 lamport rounding per match
        riskIndicator: valuationEstimate?.riskIndicator,
        unfilledQty: unfilledQty.tokens,
        hasEnoughCollateral: setupCheckEstimate && setupCheckEstimate.riskIndicator < 1 ? true : false,
        fees: feesCalc(sim.filled_vwar, totalInterest.tokens)
      });
    } catch (e) {
      console.log(e);
    }
  };

  const createBorrowOrder = async () => {
    if (!amount) return;
    setPending(true);
    let signature: string;
    try {
      if (disabled || !wallet.publicKey) return;
      signature = await borrowNow({
        market: marketAndConfig,
        marginAccount,
        provider,
        walletAddress: wallet.publicKey,
        pools: pools.tokenPools,
        amount,
        markets: markets.map(m => m.market)
      });
      setTimeout(() => {
        refreshOrderBooks();
        notify(
          'Borrow Successful',
          `Your borrow order for ${amount.div(new BN(10 ** decimals))} ${token.name} was filled successfully`,
          'success',
          getExplorerUrl(signature, cluster, explorer)
        );
        setPending(false);
      }, 2000); // TODO: Ugly and unneded, update when websocket is fully integrated
    } catch (e: any) {
      notify(
        'Borrow Order Failed',
        `Your borrow order for ${amount.div(new BN(10 ** decimals))} ${token.name} failed`,
        'error',
        getExplorerUrl(e.signature, cluster, explorer)
      );
      setPending(false);
      console.error(e);
    } finally {
      setAmount(undefined);
    }
  };

  return (
    <div className="fixed-term order-entry-body">
      <div className="borrow-now fixed-order-entry-fields">
        <label>
          Loan amount
          <InputNumber
            className="input-amount"
            onChange={debounce(e => {
              setAmount(new BN(e * 10 ** decimals));
            }, 300)}
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
                end: new Date(marketAndConfig.config.borrowTenor * 1000)
              })
            )}`}
          </span>
        </div>
        <div className="stat-line">
          <span>Repayment Amount</span>
          {forecast && (
            <span>
              {forecast.repayAmount.toFixed(token.precision)} {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Total Interest</span>
          {forecast && <span>{`~${forecast.interest.toFixed(token.precision)} ${token.symbol}`}</span>}
        </div>
        <div className="stat-line">
          <span>Interest Rate</span>
          <RateDisplay rate={forecast?.effectiveRate} />
        </div>
        <div className="stat-line">
          <span>Fees</span>
          {forecast && <span>{`~${forecast?.fees.toFixed(token.precision)} ${token.symbol}`}</span>}
        </div>
        <div className="stat-line">
          <span>Risk Indicator</span>
          {forecast && (
            <span>
              {marginAccount?.riskIndicator.toFixed(3)} → {forecast.riskIndicator?.toFixed(3)}
            </span>
          )}
        </div>
      </div>
      <Button className="submit-button" disabled={disabled || pending} onClick={createBorrowOrder}>
        {pending ? (
          <>
            <LoadingOutlined />
            Sending transaction
          </>
        ) : (
          `Borrow ${marketToString(marketAndConfig.config)}`
        )}
      </Button>
      {forecast?.selfMatch && (
        <div className="fixed-term-warning">The request would match with your own offers in this market.</div>
      )}
      {!forecast?.hasEnoughCollateral && amount && !amount.isZero() && (
        <div className="fixed-term-warning">Not enough collateral to submit this request</div>
      )}
      {forecast && forecast.unfilledQty > 0 && (
        <div className="fixed-term-warning">Not enough liquidity on this market, try a smaller amount.</div>
      )}
      {forecast && forecast.effectiveRate === 0 && (
        <div className="fixed-term-warning">Zero rate loans are not supported. Try increasing the borrow amount.</div>
      )}
    </div>
  );
};
